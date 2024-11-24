use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, IsCa,
    KeyUsagePurpose, SanType,
};
use std::net::IpAddr;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use hyper::{
    server::conn::Http,
    service::service_fn,
    Body, Request, Response,
};
use std::sync::Arc;
use rustls::{Certificate as RustlsCert, PrivateKey, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create CA certificate
    let mut ca_params = CertificateParams::new(vec!["CA".to_string()]);
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::OrganizationName, "Company, INC.");
    distinguished_name.push(rcgen::DnType::CountryName, "US");
    distinguished_name.push(rcgen::DnType::LocalityName, "San Francisco");
    ca_params.distinguished_name = distinguished_name;
    
    ca_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    
    let ca_cert = Certificate::from_params(ca_params)?;

    // Create server certificate
    let mut server_params = CertificateParams::new(vec!["localhost".to_string()]);
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(rcgen::DnType::OrganizationName, "Company, INC.");
    distinguished_name.push(rcgen::DnType::CommonName, "localhost");
    server_params.distinguished_name = distinguished_name;

    // Add localhost IP to SAN
    server_params.subject_alt_names = vec![
        SanType::IpAddress(IpAddr::V4("127.0.0.1".parse()?)),
    ];

    let server_cert = Certificate::from_params(server_params)?;
    let server_cert_signed = server_cert.serialize_pem_with_signer(&ca_cert)?;
    let server_key_pem = server_cert.serialize_private_key_pem();

    // Set up TLS configuration
    let server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    let cert_chain = vec![RustlsCert(
        rustls_pemfile::certs(&mut server_cert_signed.as_bytes())?.remove(0),
    )];
    let private_key = PrivateKey(
        rustls_pemfile::pkcs8_private_keys(&mut server_key_pem.as_bytes())?.remove(0),
    );

    let server_config = server_config.with_single_cert(cert_chain, private_key)?;
    let server_config = Arc::new(server_config);

    // Set up HTTPS server
    let addr = "127.0.0.1:8443";
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on https://{}", addr);

    async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::new(Body::from("Hello from TLS server!")))
    }

    while let Ok((stream, _)) = listener.accept().await {
        let acceptor = TlsAcceptor::from(server_config.clone());
        let tls_stream = acceptor.accept(stream).await?;
        
        tokio::spawn(async move {
            if let Err(e) = Http::new()
                .serve_connection(tls_stream, service_fn(handle_request))
                .await
            {
                eprintln!("Error serving connection: {}", e);
            }
        });
    }

    Ok(())
}