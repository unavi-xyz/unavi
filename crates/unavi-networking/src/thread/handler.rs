use anyhow::anyhow;
use bevy::{
    log::{debug, error, info, info_span},
    utils::tracing::Instrument,
};
use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use wired_world::world_server_capnp::world_server::Client;
use xwt_core::{
    base::Session,
    session::{datagram::Send, stream::OpeningBi},
};
use xwt_futures_io::{read::ReadCompat, write::WriteCompat};

use crate::thread::SessionResponse;

use super::{rpc::join::JoinError, NewSession, SessionRequest};

#[derive(Error, Debug)]
pub enum SessionError {
    #[error(transparent)]
    Capnp(#[from] capnp::Error),
    #[error("Failed to open WebTransport connection: {0}")]
    Connect(anyhow::Error),
    #[error(transparent)]
    Join(#[from] JoinError),
    #[error("Failed to open stream: {0}")]
    OpenStream(anyhow::Error),
    #[error(transparent)]
    Send(#[from] SendError<SessionResponse>),
}

pub async fn handle_session(
    NewSession {
        address,
        mut receiver,
        record_id,
        sender,
    }: NewSession,
) -> Result<(), SessionError> {
    let session = super::connect::connect(&address)
        .await
        .map_err(SessionError::Connect)?;
    info!("Started session.");

    let (writer, reader) = open_stream(&session)
        .await
        .map_err(SessionError::OpenStream)?;

    let rpc_network = VatNetwork::new(
        Box::pin(reader),
        Box::pin(writer),
        Side::Client,
        Default::default(),
    );
    let mut rpc_system = RpcSystem::new(Box::new(rpc_network), None);

    let world_server: Client = rpc_system.bootstrap(Side::Server);

    tokio::task::spawn_local(
        async move {
            match rpc_system.await {
                Ok(_) => debug!("Graceful exit."),
                Err(e) => error!("{}", e),
            };
        }
        .instrument(info_span!("RPC")),
    );
    info!("Created world server RPC.");

    super::rpc::join::join(&world_server, record_id.clone()).await?;

    let tickrate = super::rpc::tickrate::tickrate(&world_server).await?;
    sender.send(SessionResponse::Tickrate(tickrate))?;

    while let Some(action) = receiver.recv().await {
        match action {
            SessionRequest::Close => break,
            SessionRequest::SendDatagram(builder) => {
                let mut data = Vec::new();
                capnp::serialize_packed::write_message(&mut data, &builder)?;
                if let Err(e) = session.send_datagram(data).await {
                    error!("Failed to send datagram: {}", e);
                };
            }
        }
    }

    Ok(())
}

async fn open_stream<T: Session>(session: &T) -> anyhow::Result<(WriteCompat<T>, ReadCompat<T>)> {
    let (send, recv) = session
        .open_bi()
        .await
        .map_err(|e| anyhow!("{}", e))?
        .wait_bi()
        .await
        .map_err(|e| anyhow!("{}", e))?;

    let writer = WriteCompat::<T>::new(send);
    let reader = ReadCompat::<T>::new(recv);

    Ok((writer, reader))
}
