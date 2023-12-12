use std::net::SocketAddr;
use tracing::{error, info, info_span, Instrument};
use wtransport::{endpoint::IncomingSession, tls::Certificate, Endpoint, ServerConfig};

pub mod cert;

pub struct WorldOptions {
    pub address: SocketAddr,
    pub certificate: Certificate,
}

pub async fn start_server(opts: WorldOptions) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting server on {}", opts.address);

    let config = ServerConfig::builder()
        .with_bind_address(opts.address)
        .with_certificate(opts.certificate)
        .build();

    let endpoint = Endpoint::server(config)?;

    for id in 0.. {
        let incoming_session = endpoint.accept().await;
        tokio::spawn(handle_connection(incoming_session).instrument(info_span!("Connection", id)));
    }

    Ok(())
}

async fn handle_connection(incoming_session: IncomingSession) {
    if let Err(e) = handle_connection_impl(incoming_session).await {
        error!("{}", e);
    }
}

async fn handle_connection_impl(
    incoming_session: IncomingSession,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = vec![0; 65536].into_boxed_slice();

    info!("Waiting for session request...");

    let session_request = incoming_session.await?;

    info!(
        "New session: Authority: '{}', Path: '{}'",
        session_request.authority(),
        session_request.path()
    );

    let connection = session_request.accept().await?;

    info!("Waiting for data from client...");

    loop {
        tokio::select! {
            stream = connection.accept_bi() => {
                let mut stream = stream?;
                info!("Accepted BI stream");

                let bytes_read = match stream.1.read(&mut buffer).await? {
                    Some(bytes_read) => bytes_read,
                    None => continue,
                };

                let str_data = std::str::from_utf8(&buffer[..bytes_read])?;

                info!("Received (bi) '{str_data}' from client");

                stream.0.write_all(b"ACK").await?;
            }
            stream = connection.accept_uni() => {
                let mut stream = stream?;
                info!("Accepted UNI stream");

                let bytes_read = match stream.read(&mut buffer).await? {
                    Some(bytes_read) => bytes_read,
                    None => continue,
                };

                let str_data = std::str::from_utf8(&buffer[..bytes_read])?;

                info!("Received (uni) '{str_data}' from client");

                let mut stream = connection.open_uni().await?.await?;
                stream.write_all(b"ACK").await?;
            }
            dgram = connection.receive_datagram() => {
                let dgram = dgram?;
                let str_data = std::str::from_utf8(&dgram)?;

                info!("Received (dgram) '{str_data}' from client");

                connection.send_datagram(b"ACK")?;
            }
        }
    }
}