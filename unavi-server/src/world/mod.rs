use axum::http::Method;
use bytes::{BufMut, Bytes, BytesMut};
use h3::{
    error::ErrorLevel,
    ext::Protocol,
    quic::{RecvDatagramExt, SendDatagramExt, SendStreamUnframed},
    server::Connection,
};
use h3_webtransport::{server::WebTransportSession, stream};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{error, info, trace_span};

pub mod cert;

pub struct WorldOptions {
    pub address: SocketAddr,
    pub cert_pair: CertPair,
}

pub struct CertPair {
    pub cert: rustls::Certificate,
    pub key: rustls::PrivateKey,
}

pub async fn start_server(opts: WorldOptions) -> Result<(), Box<dyn std::error::Error>> {
    let mut tls_config = rustls::ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(vec![opts.cert_pair.cert], opts.cert_pair.key)?;

    tls_config.max_early_data_size = u32::MAX;
    let alpn: Vec<Vec<u8>> = vec![
        b"h3".to_vec(),
        b"h3-32".to_vec(),
        b"h3-31".to_vec(),
        b"h3-30".to_vec(),
        b"h3-29".to_vec(),
    ];
    tls_config.alpn_protocols = alpn;

    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(tls_config));
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.keep_alive_interval(Some(Duration::from_secs(2)));
    server_config.transport = Arc::new(transport_config);
    let endpoint = quinn::Endpoint::server(server_config, opts.address)?;

    println!("World server listening on {}", opts.address);

    // Accept incoming QUIC connections and spawn a new task to handle them
    while let Some(new_conn) = endpoint.accept().await {
        trace_span!("New connection being attempted");

        tokio::spawn(async move {
            match new_conn.await {
                Ok(conn) => {
                    info!("New http3 connection established");

                    let h3_conn = h3::server::builder()
                        .enable_webtransport(true)
                        .enable_connect(true)
                        .enable_datagram(true)
                        .max_webtransport_sessions(1)
                        .send_grease(true)
                        .build(h3_quinn::Connection::new(conn))
                        .await
                        .unwrap();

                    tokio::spawn(async move {
                        if let Err(err) = handle_connection(h3_conn).await {
                            error!("Failed to handle connection: {err:?}");
                        }
                    });
                }
                Err(err) => {
                    error!("Accepting connection failed: {:?}", err);
                }
            }
        });
    }

    // shut down gracefully
    // wait for connections to be closed before exiting
    endpoint.wait_idle().await;

    Ok(())
}

async fn handle_connection(
    mut conn: Connection<h3_quinn::Connection, Bytes>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        match conn.accept().await {
            Ok(Some((req, stream))) => {
                info!("New request: {:#?}", req);
                println!("New request: {:#?}", req);

                let ext = req.extensions();
                println!("Extensions: {:#?}", ext);

                println!("Method: {:#?}", req.method());

                match req.method() {
                    &Method::CONNECT if ext.get::<Protocol>() == Some(&Protocol::WEB_TRANSPORT) => {
                        println!("Initiating webtransport session");

                        let session = WebTransportSession::accept(req, stream, conn).await?;

                        println!("Established webtransport session");

                        handle_session(session).await?;

                        return Ok(());
                    }
                    _ => {
                        println!("Not a webtransport request");
                        info!(?req, "Received request");
                    }
                }
            }

            // No more streams to be received
            Ok(None) => {
                break;
            }

            Err(err) => {
                error!("Error on accept {}", err);

                match err.get_error_level() {
                    ErrorLevel::ConnectionError => break,
                    ErrorLevel::StreamError => continue,
                }
            }
        }
    }
    Ok(())
}

macro_rules! log_result {
    ($expr:expr) => {
        if let Err(err) = $expr {
            error!("{err:?}");
        }
    };
}

async fn handle_session<C>(
    session: WebTransportSession<C, Bytes>,
) -> Result<(), Box<dyn std::error::Error>>
where
    // What in the ever living fuck
    C: 'static
        + Send
        + h3::quic::Connection<Bytes>
        + RecvDatagramExt<Buf = Bytes>
        + SendDatagramExt<Bytes>,
    <C::SendStream as h3::quic::SendStream<Bytes>>::Error:
        'static + std::error::Error + Send + Sync + Into<std::io::Error>,
    <C::RecvStream as h3::quic::RecvStream>::Error:
        'static + std::error::Error + Send + Sync + Into<std::io::Error>,
    stream::BidiStream<C::BidiStream, Bytes>:
        h3::quic::BidiStream<Bytes> + Unpin + AsyncWrite + AsyncRead,
    <stream::BidiStream<C::BidiStream, Bytes> as h3::quic::BidiStream<Bytes>>::SendStream:
        Unpin + AsyncWrite + Send + Sync,
    <stream::BidiStream<C::BidiStream, Bytes> as h3::quic::BidiStream<Bytes>>::RecvStream:
        Unpin + AsyncRead + Send + Sync,
    C::SendStream: Send + Unpin,
    C::RecvStream: Send + Unpin,
    C::BidiStream: Send + Unpin,
    stream::SendStream<C::SendStream, Bytes>: AsyncWrite,
    C::BidiStream: SendStreamUnframed<Bytes>,
    C::SendStream: SendStreamUnframed<Bytes>,
{
    let session_id = session.session_id();

    // This will open a bidirectional stream and send a message to the client right after connecting!
    let stream = session.open_bi(session_id).await?;
    tokio::spawn(async move { log_result!(open_bidi_test(stream).await) });

    loop {
        tokio::select! {
            datagram = session.accept_datagram() => {
                let datagram = datagram?;
                if let Some((_, datagram)) = datagram {
                    info!("Responding with {datagram:?}");

                    let mut resp = BytesMut::from(&b"Response: "[..]);
                    resp.put(datagram);

                    session.send_datagram(resp.freeze())?;
                    info!("Finished sending datagram");
                }
            }
            uni_stream = session.accept_uni() => {
                // let (id, stream) = uni_stream?.unwrap();
                //
                // let send = session.open_uni(id).await?;
                // tokio::spawn( async move { log_result!(echo_stream(send, stream).await); });
            }
            stream = session.accept_bi() => {
                // if let Some(server::AcceptedBi::BidiStream(_, stream)) = stream? {
                //     let (send, recv) = quic::BidiStream::split(stream);
                //     tokio::spawn( async move { log_result!(echo_stream(send, recv).await); });
                // }
            }
            else => {
                break
            }
        }
    }

    info!("Finished handling session");

    Ok(())
}

async fn open_bidi_test<S>(mut stream: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: Unpin + AsyncRead + AsyncWrite,
{
    info!("Opening bidirectional stream");

    stream
        .write_all(b"Hello from a server initiated bidi stream")
        .await?;

    let mut resp = Vec::new();
    stream.shutdown().await?;
    stream.read_to_end(&mut resp).await?;

    info!("Got response from client: {resp:?}");

    Ok(())
}
