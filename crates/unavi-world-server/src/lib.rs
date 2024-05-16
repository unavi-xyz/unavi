//! Server for running multiplayer instances of worlds over WebTransport.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use tokio::task::LocalSet;
use tracing::{debug, error, info, info_span, Instrument};
use wtransport::{Endpoint, Identity, ServerConfig};
use xwt_wtransport::IncomingSession;

mod connection;
mod rpc;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub port: u16,
    pub domain: String,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), opts.port);

    let config = ServerConfig::builder()
        .with_bind_address(address)
        .with_identity(&Identity::self_signed(["localhost", "127.0.0.1", &opts.domain]).unwrap())
        .build();

    let endpoint = Endpoint::server(config)?;
    let endpoint = xwt_wtransport::Endpoint(endpoint);

    let num_threads = std::thread::available_parallelism().unwrap().into();
    debug!("Spawning {} connection threads.", num_threads);

    let mut threads = Vec::new();

    for thread in 0..num_threads {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<NewConnection>();
        threads.push(sender);

        std::thread::spawn(move || {
            let _ = info_span!("Thread", id = thread).enter();

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let local = LocalSet::new();

            local.spawn_local(async move {
                while let Some(new_connection) = receiver.recv().await {
                    debug!("Handling connection {}", new_connection.id);

                    let span = info_span!("Connection", id = new_connection.id);

                    tokio::task::spawn_local(
                        connection::handle_connection(new_connection.incoming_session)
                            .instrument(span),
                    );
                }
            });

            rt.block_on(local);

            debug!("Thread finished.")
        });
    }

    info!("Listening on {}", address);

    for id in 1.. {
        let incoming_session = IncomingSession(endpoint.accept().await);

        let thread_idx = num_threads % id;

        if let Err(e) = threads[thread_idx].send(NewConnection {
            id,
            incoming_session,
        }) {
            error!("{}", e);
        };

        debug!("Connection {} sent to thread {}.", id, thread_idx)
    }

    info!("Finished.");
    Ok(())
}

struct NewConnection {
    id: usize,
    incoming_session: IncomingSession,
}
