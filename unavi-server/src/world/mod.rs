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
use tokio::io::{AsyncRead, AsyncWrite};
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
    let tls_config = rustls::ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(vec![opts.cert_pair.cert], opts.cert_pair.key)?;

    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(tls_config));
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.keep_alive_interval(Some(Duration::from_secs(2)));
    server_config.transport = Arc::new(transport_config);

    let endpoint = quinn::Endpoint::server(server_config, opts.address)?;

    println!("World server listening on {}", opts.address);

    while let Some(new_conn) = endpoint.accept().await {
        println!("New connection being attempted");
        trace_span!("New connection being attempted");

        tokio::spawn(async move {
            match new_conn.await {
                Ok(conn) => {
                    info!("http3 connection established");

                    let h3_conn = h3::server::builder()
                        .enable_webtransport(true)
                        .enable_connect(true)
                        .enable_datagram(true)
                        .max_webtransport_sessions(1)
                        .send_grease(true)
                        .build(h3_quinn::Connection::new(conn))
                        .await
                        .unwrap();

                    // tracing::info!("Establishing WebTransport session");
                    // // 3. TODO: Conditionally, if the client indicated that this is a webtransport session, we should accept it here, else use regular h3.
                    // // if this is a webtransport session, then h3 needs to stop handing the datagrams, bidirectional streams, and unidirectional streams and give them
                    // // to the webtransport session.

                    tokio::spawn(async move {
                        if let Err(err) = handle_connection(h3_conn).await {
                            tracing::error!("Failed to handle connection: {err:?}");
                        }
                    });

                    // let mut session: WebTransportSession<_, Bytes> =
                    //     WebTransportSession::accept(h3_conn).await.unwrap();
                    // tracing::info!("Finished establishing webtransport session");
                    // // 4. Get datagrams, bidirectional streams, and unidirectional streams and wait for client requests here.
                    // // h3_conn needs to handover the datagrams, bidirectional streams, and unidirectional streams to the webtransport session.
                    // let result = handle.await;
                }
                Err(err) => {
                    error!("accepting connection failed: {:?}", err);
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
    // 3. TODO: Conditionally, if the client indicated that this is a webtransport session, we should accept it here, else use regular h3.
    // if this is a webtransport session, then h3 needs to stop handing the datagrams, bidirectional streams, and unidirectional streams and give them
    // to the webtransport session.

    loop {
        match conn.accept().await {
            Ok(Some((req, stream))) => {
                info!("new request: {:#?}", req);

                let ext = req.extensions();
                match req.method() {
                    &Method::CONNECT if ext.get::<Protocol>() == Some(&Protocol::WEB_TRANSPORT) => {
                        tracing::info!("Peer wants to initiate a webtransport session");

                        tracing::info!("Handing over connection to WebTransport");
                        let session = WebTransportSession::accept(req, stream, conn).await?;
                        tracing::info!("Established webtransport session");
                        // 4. Get datagrams, bidirectional streams, and unidirectional streams and wait for client requests here.
                        // h3_conn needs to handover the datagrams, bidirectional streams, and unidirectional streams to the webtransport session.
                        handle_session(session).await?;

                        return Ok(());
                    }
                    _ => {
                        tracing::info!(?req, "Received request");
                    }
                }
            }

            // indicating no more streams to be received
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
    let _stream = session.open_bi(session_id).await?;

    loop {
        tokio::select! {
            datagram = session.accept_datagram() => {
                let datagram = datagram?;
                if let Some((_, datagram)) = datagram {
                    tracing::info!("Responding with {datagram:?}");
                    // Put something before to make sure encoding and decoding works and don't just
                    // pass through
                    let mut resp = BytesMut::from(&b"Response: "[..]);
                    resp.put(datagram);


                    session.send_datagram(resp.freeze())?;
                    tracing::info!("Finished sending datagram");
                }
            }
            uni_stream = session.accept_uni() => {
                let (id, _stream) = uni_stream?.unwrap();

                let _send = session.open_uni(id).await?;
            }
            stream = session.accept_bi() => {
                if let Some(h3_webtransport::server::AcceptedBi::BidiStream(_, stream)) = stream? {
                    let (_send, _recv) = h3::quic::BidiStream::split(stream);
                }
            }
            else => {
                break
            }
        }
    }

    tracing::info!("Finished handling session");

    Ok(())
}
