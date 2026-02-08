//! Inbound object stream handler - receives physics state from remote owners.
//!
//! Object ID is known from `StreamInit::Object` sent by `SpaceProtocol`.
//! I-frames are received on the dedicated bistream.
//! P-frames are received as datagrams (handled in `SpaceProtocol`).

use bevy::log::debug;
use iroh::EndpointId;

use crate::networking::thread::{
    NetworkEvent,
    space::{msg::ObjectIFrameMsg, types::object_id::ObjectId},
};

/// Handle a single object's dedicated bistream.
/// Object ID is known from `StreamInit::Object`, passed by caller.
pub async fn handle_object_stream(
    event_tx: tokio::sync::mpsc::Sender<NetworkEvent>,
    source: EndpointId,
    object_id: ObjectId,
    _send: iroh::endpoint::SendStream,
    mut recv: iroh::endpoint::RecvStream,
) -> anyhow::Result<()> {
    debug!(?object_id, "object stream initialized");

    // Receive I-frames for this object.
    loop {
        let mut len_buf = [0u8; 4];
        if recv.read_exact(&mut len_buf).await.is_err() {
            break;
        }

        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        recv.read_exact(&mut buf).await?;

        let msg: ObjectIFrameMsg = postcard::from_bytes(&buf)?;
        debug!(id = msg.id, ?object_id, "received object I-frame");

        // Send event with the received physics state.
        let _ = event_tx.try_send(NetworkEvent::ObjectPoseUpdate {
            source,
            objects: vec![(object_id, msg.state)],
        });
    }

    Ok(())
}
