
<div align="center">

  <a href="https://github.com/mranv/rust-cert-authority/blob/singed-cert/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </a>
  <!-- <a href="https://github.com/mranv/rust-cert-authority/actions">
    <img src="https://github.com/mranv/rust-cert-authority/workflows/CI/badge.svg" alt="CI">
  </a> -->
  <!-- <a href="https://crates.io/crates/rustls-ca-forge">
    <img src="https://img.shields.io/crates/v/rustls-ca-forge.svg" alt="Version">
  </a>
  <a href="https://docs.rs/rustls-ca-forge">
    <img src="https://docs.rs/rustls-ca-forge/badge.svg" alt="Documentation">
  </a> -->

  <br>
<img src=assets/image.png>
<br>

  <h3> rustls-ca-forge : A Modern Certificate Authority and TLS Certificate Generator</h3>

  <p>Create and manage certificates with the power of Rust 🦀</p>
</div>

<div align="center">
  <img src="/api/placeholder/800/400" alt="rustls-ca-forge banner">
</div>

## ✨ Features

- 🛡️ **Secure by Default**: Built with Rust's safety guarantees
- 🚀 **High Performance**: Async I/O with tokio
- 📦 **Zero Config**: Works out of the box
- 🔄 **Auto Renewal**: Automatic certificate management
- 🎯 **Type Safe**: Strongly typed certificate handling

## 🚀 Quick Start

```bash
# Install via cargo
cargo install rustls-ca-forge

# Generate a new CA certificate
rustls-ca-forge init

# Create a server certificate
rustls-ca-forge create-cert
```

## 💡 Usage

```rust
use rustls_ca_forge::{CertificateAuthority, ServerCertificate};

#[tokio::main]
async fn main() {
    // Create a new CA
    let ca = CertificateAuthority::new("My Root CA");
    
    // Generate server certificate
    let cert = ServerCertificate::new("localhost")
        .sign_with(&ca)
        .await;
}
```

## 🛠️ Development

<details>
<summary>Click to expand</summary>

```bash
# Clone the repository
git clone https://github.com/mranv/rust-cert-authority
cd rust-cert-authority

# Build
cargo build

# Run tests
cargo test

# Start the server
cargo run
```
</details>

## 📋 Requirements

- Rust 1.70.0 or higher
- OpenSSL development packages
- Linux, macOS, or Windows


## 📄 License

<div align="center">

  **rustls-ca-forge** © [Anubhav Gain](https://github.com/mranv). Released under the [MIT License](LICENSE).<br>
  Authored and maintained by Anubhav Gain.

  > [iamanubhavgain@gmail.com](mailto:iamanubhavgain@gmail.com) · GitHub [@mranv](https://github.com/mranv)

</div>

---

<div align="center">
  <sub>Built with ❤️ by <a href="https://mranv.github.io">Anubhav Gain</a></sub>
</div>