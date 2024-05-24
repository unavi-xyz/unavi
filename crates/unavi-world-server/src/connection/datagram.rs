use std::sync::Arc;

use capnp::message::ReaderOptions;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use wired_world::datagram_capnp;
use xwt_wtransport::Datagram;

use crate::{
    commands::{SessionCommand, SessionMessage, Transform},
    global_context::GlobalContext,
};

#[derive(Error, Debug)]
pub enum HandleDiagramError {
    #[error(transparent)]
    Capnp(#[from] capnp::Error),
    #[error(transparent)]
    Send(#[from] SendError<SessionMessage>),
}

pub async fn handle_datagram(
    player_id: usize,
    context: Arc<GlobalContext>,
    dgram: Datagram,
) -> Result<(), HandleDiagramError> {
    let message = capnp::serialize_packed::read_message(dgram.as_ref(), ReaderOptions::default())?;

    let publish_transform = message.get_root::<datagram_capnp::publish_transform::Reader>()?;
    let transform = publish_transform.get_transform()?;

    let translation = transform.get_translation()?;
    let translation = [
        translation.get_x(),
        translation.get_y(),
        translation.get_z(),
    ];

    let rotation = transform.get_rotation()?;
    let rotation = [
        rotation.get_x(),
        rotation.get_y(),
        rotation.get_z(),
        rotation.get_w(),
    ];

    context.sender.send(SessionMessage {
        command: SessionCommand::SetTransform(Transform {
            translation,
            rotation,
        }),
        player_id,
    })?;

    Ok(())
}
