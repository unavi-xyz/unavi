use anyhow::Result;
use tracing::{error, info, info_span, Instrument};
use xwt_core::{
    endpoint::accept::{Accepting, Request},
    session::{datagram::Receive, stream::AcceptBi},
};

use crate::NewConnection;

mod bi_stream;
mod datagram;

pub async fn handle_connection(new_connection: NewConnection) {
    if let Err(e) = handle_connection_impl(new_connection).await {
        error!("{}", e);
    }
}

async fn handle_connection_impl(new_connection: NewConnection) -> Result<()> {
    info!("Waiting for session request...");
    let session_request = new_connection.incoming_session.wait_accept().await?;

    info!(
        "New session: Authority: '{}', Path: '{}'",
        session_request.0.authority(),
        session_request.0.path()
    );
    let session = session_request.ok().await?;

    info!("Waiting for data from client...");

    loop {
        tokio::select! {
            stream = session.accept_bi() => {
                let stream = stream?;
                info!("Accepted bi stream");
                tokio::task::spawn_local(
                    bi_stream::handle_bi_stream(new_connection.id, stream).instrument(info_span!("bi"))
                );
            }
            dgram = session.receive_datagram() => {
                let dgram = dgram?;
                datagram::handle_datagram(dgram, &session).instrument(info_span!("dgram")).await?;
            }
        }
    }
}
