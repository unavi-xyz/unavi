use std::sync::Arc;

use anyhow::{anyhow, Result};
use dwn::Dwn;
use tracing::{error, info, info_span, Instrument};

use xwt_core::{
    endpoint::accept::{Accepting, Request},
    session::{datagram::Receive, stream::AcceptBi},
};

use crate::{
    global_context::GlobalContext,
    update_loop::{IncomingCommand, IncomingEvent},
    NewConnection,
};

mod bi_stream;
mod datagram;
mod event;

pub async fn handle_connection(
    new_connection: NewConnection,
    context: Arc<GlobalContext>,
    dwn: Arc<Dwn>,
) -> Result<()> {
    let player_id = new_connection.id;

    if let Err(e) = handle_connection_impl(new_connection, context.clone(), dwn).await {
        error!("Connection failed: {}", e);
    }

    context.sender.send(IncomingEvent {
        command: IncomingCommand::Disconnect,
        player_id,
    })?;

    Ok(())
}

async fn handle_connection_impl(
    new_connection: NewConnection,
    context: Arc<GlobalContext>,
    dwn: Arc<Dwn>,
) -> Result<()> {
    info!("Waiting for session request...");
    let session_request = new_connection.incoming_session.wait_accept().await?;

    info!(
        "New session: Authority: '{}', Path: '{}'",
        session_request.0.authority(),
        session_request.0.path()
    );
    let session = session_request.ok().await?;

    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

    context.sender.send(IncomingEvent {
        player_id: new_connection.id,
        command: IncomingCommand::NewPlayer { sender },
    })?;

    let mut event_context = event::EventContext::default();

    loop {
        let context = context.clone();
        let dwn = dwn.clone();

        tokio::select! {
            event = receiver.recv() => {
                let event = event.ok_or(anyhow!("Event channel closed"))?;
                event::handle_event(event, &mut event_context, &session).await?;
            }
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
