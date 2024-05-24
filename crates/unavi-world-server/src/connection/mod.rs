use std::sync::Arc;

use anyhow::Result;
use dwn::{
    store::{DataStore, MessageStore},
    DWN,
};
use tracing::{error, info, info_span, Instrument};
use xwt_core::{
    endpoint::accept::{Accepting, Request},
    session::{datagram::Receive, stream::AcceptBi},
};

use crate::{
    commands::{SessionCommand, SessionMessage},
    global_context::GlobalContext,
    NewConnection,
};

mod bi_stream;
mod datagram;

pub async fn handle_connection<D: DataStore + 'static, M: MessageStore + 'static>(
    new_connection: NewConnection,
    context: Arc<GlobalContext>,
    dwn: Arc<DWN<D, M>>,
) -> Result<()> {
    let player_id = new_connection.id;

    if let Err(e) = handle_connection_impl(new_connection, context.clone(), dwn).await {
        error!("Connection failed: {}", e);
    }

    context.sender.send(SessionMessage {
        command: SessionCommand::Disconnect,
        player_id,
    })?;

    Ok(())
}

async fn handle_connection_impl<D: DataStore + 'static, M: MessageStore + 'static>(
    new_connection: NewConnection,
    context: Arc<GlobalContext>,
    dwn: Arc<DWN<D, M>>,
) -> Result<()> {
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
        let context = context.clone();
        let dwn = dwn.clone();

        tokio::select! {
            stream = session.accept_bi() => {
                let stream = stream?;
                info!("Accepted bi stream.");
                tokio::task::spawn_local(
                    bi_stream::handle_bi_stream(new_connection.id, context, dwn, stream).instrument(info_span!("bi"))
                );
            }
            dgram = session.receive_datagram() => {
                let dgram = dgram?;
                datagram::handle_datagram(new_connection.id, context, dgram).instrument(info_span!("dgram")).await?;
            }
        }
    }
}
