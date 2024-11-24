use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, IsCa,
    KeyUsagePurpose, SanType, RcgenError, DnType,
};
use std::{
    net::IpAddr,
    fs::File,
    io::Write,
    path::Path,
    error::Error,
    fmt,
};
use time::OffsetDateTime;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use hyper::{
    server::conn::Http,
    service::service_fn,
    Body, Request, Response,
};
use std::sync::Arc;
use rustls::{Certificate as RustlsCert, PrivateKey, ServerConfig};

#[derive(Debug)]
enum CertError {
    Io(std::io::Error),
    Rcgen(RcgenError),
}

impl fmt::Display for CertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CertError::Io(err) => write!(f, "IO error: {}", err),
            CertError::Rcgen(err) => write!(f, "Certificate generation error: {}", err),
        }
    }
}

impl Error for CertError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CertError::Io(err) => Some(err),
            CertError::Rcgen(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for CertError {
    fn from(error: std::io::Error) -> Self {
        CertError::Io(error)
    }
}

impl From<RcgenError> for CertError {
    fn from(error: RcgenError) -> Self {
        CertError::Rcgen(error)
    }
}

async fn save_certificates(
    ca_cert: &Certificate,
    server_cert: &Certificate,
    server_cert_signed: &str,
) -> Result<(), CertError> {
    let cert_dir = Path::new("certs");
    std::fs::create_dir_all(cert_dir)?;

    let mut ca_file = File::create(cert_dir.join("ca.crt"))?;
    ca_file.write_all(ca_cert.serialize_pem()?.as_bytes())?;
    println!("CA certificate saved to certs/ca.crt");

    let mut server_cert_file = File::create(cert_dir.join("server.crt"))?;
    server_cert_file.write_all(server_cert_signed.as_bytes())?;
    println!("Server certificate saved to certs/server.crt");

    let mut server_key_file = File::create(cert_dir.join("server.key"))?;
    server_key_file.write_all(server_cert.serialize_private_key_pem().as_bytes())?;
    println!("Server private key saved to certs/server.key");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up CA certificate
    let mut ca_params = CertificateParams::new(vec![]);
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::OrganizationName, "My Root CA");
    distinguished_name.push(DnType::CommonName, "My Root CA");
    ca_params.distinguished_name = distinguished_name;
    
    // Set validity period
    let now = OffsetDateTime::now_utc();
    let one_year = time::Duration::days(365);
    ca_params.not_before = now;
    ca_params.not_after = now + one_year;
    
    ca_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::CrlSign,
    ];
    
    let ca_cert = Certificate::from_params(ca_params)?;

    // Set up server certificate
    let mut server_params = CertificateParams::new(vec![]);
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::OrganizationName, "My Server");
    distinguished_name.push(DnType::CommonName, "localhost");
    server_params.distinguished_name = distinguished_name;

    // Set validity period for server cert
    server_params.not_before = now;
    server_params.not_after = now + one_year;

    // Add both localhost and IP to SAN
    server_params.subject_alt_names = vec![
        SanType::DnsName("localhost".to_string()),
        SanType::DnsName("127.0.0.1".to_string()),
        SanType::IpAddress(IpAddr::V4("127.0.0.1".parse()?)),
    ];

    let server_cert = Certificate::from_params(server_params)?;
    let server_cert_signed = server_cert.serialize_pem_with_signer(&ca_cert)?;
    
    // Save certificates
    save_certificates(&ca_cert, &server_cert, &server_cert_signed).await?;

    // Set up TLS configuration
    let cert_chain = vec![RustlsCert(
        rustls_pemfile::certs(&mut server_cert_signed.as_bytes())?.remove(0),
    )];
    let private_key = PrivateKey(
        rustls_pemfile::pkcs8_private_keys(&mut server_cert.serialize_private_key_pem().as_bytes())?.remove(0),
    );

    let server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)?;
    let server_config = Arc::new(server_config);

    // Set up HTTPS server
    let addr = "127.0.0.1:8443";
    let listener = TcpListener::bind(addr).await?;
    println!("\nServer listening on https://{}", addr);
    println!("\nTo test with curl (in another terminal):");
    println!("curl --cacert certs/ca.crt https://127.0.0.1:8443");

    async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::new(Body::from("Hello from secure TLS server!")))
    }

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let acceptor = TlsAcceptor::from(server_config.clone());
            
            tokio::spawn(async move {
                match acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        if let Err(e) = Http::new()
                            .serve_connection(tls_stream, service_fn(handle_request))
                            .await
                        {
                            eprintln!("Error serving connection: {}", e);
                        }
                    },
                    Err(e) => eprintln!("TLS Error: {}", e),
                }
            });
        }
    }
}