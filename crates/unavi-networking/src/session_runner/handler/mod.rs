use anyhow::anyhow;
use bevy::{
    log::{debug, error, info, info_span},
    utils::tracing::Instrument,
};
use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};
use thiserror::Error;
use wired_world::world_server_capnp::world_server::Client;
use xwt_core::{
    base::Session,
    session::{datagram::Send, stream::OpeningBi},
};
use xwt_futures_io::{read::ReadCompat, write::WriteCompat};

use super::{InstanceAction, NewSession};

mod join_instance;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error(transparent)]
    Capnp(#[from] capnp::Error),
    #[error("Failed to open WebTransport connection: {0}")]
    Connect(anyhow::Error),
    #[error(transparent)]
    JoinInstance(#[from] join_instance::JoinInstanceError),
    #[error("Failed to open stream: {0}")]
    OpenStream(anyhow::Error),
}

pub async fn handle_session(
    NewSession {
        address,
        receiver: mut action_receiver,
        record_id,
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

    join_instance::join_instance(&world_server, record_id).await?;

    while let Some(action) = action_receiver.recv().await {
        debug!("Handling action: {:?}", action);

        match action {
            InstanceAction::Close => break,
            InstanceAction::SendDatagram(data) => {
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
