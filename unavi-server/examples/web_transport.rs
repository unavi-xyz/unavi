use axum::http::Uri;
use std::net::SocketAddr;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use unavi_server::world::{cert::new_ca, CertPair, WorldOptions};
use wtransport::Endpoint;

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .with_env_filter(env_filter)
        .init();

    let address = SocketAddr::from(([127, 0, 0, 1], 4433));
    let domain = "localhost".to_string();

    // Generate a self-signed certificate
    let ca = new_ca();
    let cert = rcgen::generate_simple_self_signed(vec![domain.clone()]).unwrap();
    let key = cert.serialize_private_key_der();
    let cert = cert.serialize_der_with_signer(&ca).unwrap();
    let ca = rustls::Certificate(ca.serialize_der().unwrap());

    let opts = WorldOptions {
        address,
        cert_pair: CertPair { cert, key },
    };

    // Start the server
    tokio::spawn(async move {
        if let Err(e) = unavi_server::world::start_server(opts).await {
            tracing::error!("Server: {}", e);
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Connect to server
    let uri = Uri::builder()
        .scheme("https")
        .authority(format!("{}:{}", domain, address.port()))
        .path_and_query("/")
        .build()
        .unwrap();

    if let Err(e) = connect(uri, &ca).await {
        tracing::error!("Client: {}", e);
    }
}

async fn connect(uri: Uri, ca: &rustls::Certificate) -> Result<(), Box<dyn std::error::Error>> {
    // Load CA certificates stored in the system
    let mut roots = rustls::RootCertStore::empty();

    match rustls_native_certs::load_native_certs() {
        Ok(certs) => {
            for cert in certs {
                if let Err(e) = roots.add(&rustls::Certificate(cert.0)) {
                    Err(e)?;
                }
            }
        }
        Err(e) => {
            Err(e)?;
        }
    };

    // Load certificate of CA who issues the server certificate
    // NOTE that this should be used for dev only
    if let Err(e) = roots.add(ca) {
        Err(e)?;
    }

    let mut tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();
    tls_config.enable_early_data = true;
    tls_config.alpn_protocols = vec![b"h3".into()];

    let config = wtransport::ClientConfig::builder()
        .with_bind_default()
        .with_custom_tls(tls_config)
        .build();

    let connection = Endpoint::client(config)?.connect(uri.to_string()).await?;

    let mut stream = connection.open_bi().await.unwrap().await.unwrap();
    stream.0.write_all(b"HELLO").await.unwrap();
    stream.0.finish().await.unwrap();

    Ok(())
}
