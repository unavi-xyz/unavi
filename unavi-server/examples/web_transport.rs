use axum::http::{Method, Uri};
use std::{net::SocketAddr, sync::Arc};
use tokio::io::AsyncWriteExt;
use unavi_server::world::{cert::new_ca, CertPair, WorldOptions};

#[tokio::main]
async fn main() {
    let address = SocketAddr::from(([127, 0, 0, 1], 4433));
    let domain = "localhost".to_string();

    // Generate a self-signed certificate
    let ca = new_ca();
    let cert = rcgen::generate_simple_self_signed(vec![domain.clone()]).unwrap();
    let key = rustls::PrivateKey(cert.serialize_private_key_der());
    let cert = rustls::Certificate(cert.serialize_der_with_signer(&ca).unwrap());
    let ca = rustls::Certificate(ca.serialize_der().unwrap());

    let opts = WorldOptions {
        address,
        cert_pair: CertPair { cert, key },
    };

    // Start the server
    tokio::spawn(async move {
        match unavi_server::world::start_server(opts).await {
            Ok(_) => println!("Server exited"),
            Err(e) => panic!("Server: {}", e),
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Connect to server
    let uri = Uri::builder()
        .scheme("https")
        .authority(format!("{}:{}", domain, address.port()))
        .path_and_query("/")
        .build()
        .unwrap();

    match connect(uri, &ca).await {
        Ok(_) => println!("Client exited"),
        Err(e) => panic!("Client: {}", e),
    }
}

async fn connect(uri: Uri, ca: &rustls::Certificate) -> Result<(), Box<dyn std::error::Error>> {
    if uri.scheme() != Some(&axum::http::uri::Scheme::HTTPS) {
        Err("URI scheme must be 'https'")?;
    }

    let auth = uri.authority().ok_or("URI must have a host")?.clone();
    let port = auth.port_u16().unwrap_or(443);

    let addr = match auth.host() {
        "localhost" => SocketAddr::from(([127, 0, 0, 1], port)),
        host => tokio::net::lookup_host((host, port))
            .await?
            .next()
            .ok_or("DNS found no addresses")?,
    };

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

    let mut client_endpoint = h3_quinn::quinn::Endpoint::client("[::]:0".parse().unwrap())?;

    let client_config = quinn::ClientConfig::new(Arc::new(tls_config));
    client_endpoint.set_default_client_config(client_config);

    println!("Connecting to {} at {}", auth.host(), addr);

    let conn = client_endpoint.connect(addr, auth.host())?.await?;

    println!("QUIC connection established");

    let quinn_conn = h3_quinn::Connection::new(conn);
    let (mut driver, mut send_request) = h3::client::new(quinn_conn).await?;

    let drive = async move {
        std::future::poll_fn(|cx| driver.poll_close(cx)).await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    };

    let request = async move {
        println!("Sending request...");

        let req = axum::http::Request::builder()
            .method(Method::CONNECT)
            .uri(uri)
            .body(())?;

        // Sending request results in a bidirectional stream, which is also used for receiving response
        let mut stream = send_request.send_request(req).await?;

        // Finish on the sending side
        stream.finish().await?;

        println!("Receiving response...");

        let resp = stream.recv_response().await?;

        println!("Response: {:?} {}", resp.version(), resp.status());
        println!("Headers: {:#?}", resp.headers());

        // `recv_data()` must be called after `recv_response()` for
        // receiving potential response body
        while let Some(mut chunk) = stream.recv_data().await? {
            let mut out = tokio::io::stdout();
            out.write_all_buf(&mut chunk).await?;
            out.flush().await?;
        }

        Ok::<_, Box<dyn std::error::Error>>(())
    };

    let (req_res, drive_res) = tokio::join!(request, drive);
    req_res?;
    drive_res?;

    // wait for the connection to be closed before exiting
    client_endpoint.wait_idle().await;

    Ok(())
}
