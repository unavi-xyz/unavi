//! Server for running multiplayer worlds over WebTransport.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use dwn::Dwn;
use tokio::task::LocalSet;
use tracing::{debug, error, info, info_span, Instrument};
use wtransport::{Identity, ServerConfig};
use xwt_wtransport::IncomingSession;

use crate::global_context::GlobalContext;

mod connection;
mod global_context;
mod rpc;
mod update_loop;

#[derive(Clone)]
pub struct ServerOptions {
    pub domain: String,
    pub dwn: Arc<Dwn>,
    pub port: u16,
    pub threads: Option<usize>,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), opts.port);

    let config = ServerConfig::builder()
        .with_bind_address(address)
        .with_identity(&Identity::self_signed([&address.to_string(), &opts.domain]).unwrap())
        .build();

    let endpoint = wtransport::Endpoint::server(config)?;
    let endpoint = xwt_wtransport::Endpoint(endpoint);

    let (send_cmd, recv_cmd) = tokio::sync::mpsc::unbounded_channel();

    let context = Arc::new(GlobalContext {
        sender: send_cmd.clone(),
        world_host_did: format!("did:web:{}", opts.domain.clone().replace(':', "%3A")),
    });

    let max_threads = std::thread::available_parallelism().unwrap().into();
    let num_threads = opts
        .threads
        .map(|t| t.min(max_threads))
        .unwrap_or(max_threads);
    debug!("Spawning {} connection threads.", num_threads);

    let mut threads = Vec::new();

    for thread in 0..num_threads {
        let (send_conn, mut recv_conn) = tokio::sync::mpsc::unbounded_channel::<NewConnection>();
        threads.push(send_conn);

        let context = context.clone();
        let dwn = opts.dwn.clone();

        std::thread::spawn(move || {
            let span_thread = info_span!("Thread", id = thread).entered();

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let local = LocalSet::new();

            local.spawn_local(async move {
                while let Some(new_connection) = recv_conn.recv().await {
                    debug!("Handling connection {}", new_connection.id);

                    let span = info_span!("Connection", id = new_connection.id);
                    let context = context.clone();
                    let dwn = dwn.clone();

                    tokio::task::spawn_local(
                        connection::handle_connection(new_connection, context, dwn)
                            .instrument(span),
                    );
                }
            });

            rt.block_on(local);

            debug!("Thread finished.");
            span_thread.exit();
        });
    }

    tokio::spawn(async move {
        if let Err(e) = update_loop::update_loop(send_cmd, recv_cmd).await {
            panic!("{}", e);
        };
    });

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
