use anyhow::anyhow;
use bevy::{
    log::{debug, error, info, info_span},
    utils::tracing::Instrument,
};
use capnp::message::ReaderOptions;
use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};
use thiserror::Error;
use tokio::sync::mpsc::{error::SendError, UnboundedSender};
use wired_world::{datagram_capnp, world_server_capnp::world_server::Client};
use xwt_core::{
    base::Session,
    session::{datagram::Receive, stream::OpeningBi},
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
    #[error("Connection failed: {0}")]
    Connection(anyhow::Error),
    #[error("Event channel closed")]
    EventChannelClosed,
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

    loop {
        tokio::select! {
            datagram = session.receive_datagram() => {
                let datagram = datagram.map_err(|e| SessionError::Connection(anyhow!("{}", e)))?;
                handle_datagram(datagram,  &sender).await?;
            }
            event = receiver.recv() => {
                let event = event.ok_or(SessionError::EventChannelClosed)?;
                let closed = handle_event(event, &session).await?;
                if closed {
                    break;
                }
            }
        };
    }

    Ok(())
}

async fn handle_datagram(
    dgram: impl AsRef<[u8]>,
    sender: &UnboundedSender<SessionResponse>,
) -> Result<(), SessionError> {
    let msg = capnp::serialize_packed::read_message(dgram.as_ref(), ReaderOptions::default())?;
    let transform = msg.get_root::<datagram_capnp::receive_transform::Reader>()?;

    let player = transform.get_player_id();

    let rotation = transform.get_rotation()?;
    let rotation = [
        rotation.get_x(),
        rotation.get_y(),
        rotation.get_z(),
        rotation.get_w(),
    ];

    let translation = transform.get_translation()?;
    let translation = [
        translation.get_x(),
        translation.get_y(),
        translation.get_z(),
    ];

    sender.send(SessionResponse::PlayerTransform {
        player,
        rotation,
        translation,
    })?;

    Ok(())
}

async fn handle_event(event: SessionRequest, session: &impl Session) -> Result<bool, SessionError> {
    match event {
        SessionRequest::Close => return Ok(true),
        SessionRequest::SendDatagram(builder) => {
            let mut data = Vec::new();
            capnp::serialize_packed::write_message(&mut data, &builder)?;
            if let Err(e) = session.send_datagram(data).await {
                error!("Failed to send datagram: {}", e);
            };
        }
    };

    return Ok(false);
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
