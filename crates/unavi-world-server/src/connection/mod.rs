use anyhow::Result;
use tracing::{error, info, info_span, Instrument};
use xwt_core::{
    endpoint::accept::{Accepting, Request},
    session::{datagram::Receive, stream::AcceptBi},
};
use xwt_wtransport::IncomingSession;

mod bi_stream;
mod datagram;

pub async fn handle_connection(incoming_session: IncomingSession) {
    if let Err(e) = handle_connection_impl(incoming_session).await {
        error!("{}", e);
    }
}

async fn handle_connection_impl(incoming_session: IncomingSession) -> Result<()> {
    info!("Waiting for session request...");
    let session_request = incoming_session.wait_accept().await?;

    info!(
        "New session: Authority: '{}', Path: '{}'",
        session_request.0.authority(),
        session_request.0.path()
    );
    let connection = session_request.ok().await?;

    info!("Waiting for data from client...");

    loop {
        tokio::select! {
            stream = connection.accept_bi() => {
                let stream = stream?;
                info!("Accepted BI stream");

                 tokio::task::spawn_local(bi_stream::handle_bi_stream(stream).instrument(info_span!("bi")));
            }
            dgram = connection.receive_datagram() => {
                let dgram = dgram?;
                datagram::handle_datagram(dgram, &connection).instrument(info_span!("dgram")).await?;
            }
        }
    }
}
