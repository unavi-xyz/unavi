//! Outbound object stream handler - sends physics state to remote peers.
//!
//! For each claimed object, opens a dedicated bistream on the existing connection:
//! 1. Send `StreamInit::Object { object_id }` to identify the stream
//! 2. Send I-frames reliably on the stream (without `object_id`, it's known from init)
//! 3. Send P-frames as datagrams (tagged with object ID for routing)

use std::sync::{Arc, atomic::AtomicU8};

use bevy::log::debug;
use bytes::Bytes;
use iroh::{EndpointId, endpoint::Connection};

use crate::networking::thread::space::{
    MAX_TICKRATE,
    buffer::{DATAGRAM_MAX_SIZE, OBJECT_IFRAME_MSG_MAX_SIZE},
    msg::{Datagram, ObjectIFrameMsg, ObjectPFrameDatagram, StreamInit},
    types::object_id::ObjectId,
};

use super::super::agent::outbound::send_stream_init;

/// State for a single object's outbound stream.
///
/// Each object stream has its own tickrate that can be adjusted based on
/// distance. The receiver calculates distance to the object and sends
/// `ControlMsg::TickrateRequest` to adjust the update rate.
pub struct ObjectStreamState {
    pub object_id: ObjectId,
    /// Current tickrate for this object stream.
    /// Receiver can request lower tickrate for distant objects.
    pub tickrate: Arc<AtomicU8>,
    pub iframe_id: u16,
    pub pframe_seq: u16,
}

impl ObjectStreamState {
    pub fn new(object_id: ObjectId) -> Self {
        Self {
            object_id,
            tickrate: Arc::new(AtomicU8::new(MAX_TICKRATE)),
            iframe_id: 0,
            pframe_seq: 0,
        }
    }

    /// Update tickrate based on distance-based request.
    ///
    /// The effective tickrate is the minimum of local preference and remote request.
    #[expect(unused)]
    pub fn update_tickrate(&self, requested_hz: u8) {
        use std::sync::atomic::Ordering;
        let effective = requested_hz.min(MAX_TICKRATE);
        self.tickrate.store(effective, Ordering::Relaxed);
    }
}

/// Open a new object stream for a claimed object on an existing connection.
pub async fn open_object_stream(
    connection: &Connection,
    object_id: ObjectId,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] peer: EndpointId,
) -> anyhow::Result<(iroh::endpoint::SendStream, iroh::endpoint::RecvStream)> {
    debug!(?object_id, ?peer, "opening object stream");

    let (mut send, recv) = connection.open_bi().await?;

    // Send stream init with object ID.
    send_stream_init(&mut send, &StreamInit::Object { object_id }).await?;

    debug!(?object_id, ?peer, "object stream opened");

    Ok((send, recv))
}

/// Send an object I-frame on its dedicated stream.
/// Object ID is not included - it's known from the `StreamInit` sent at stream creation.
pub async fn send_object_iframe(
    stream: &mut iroh::endpoint::SendStream,
    msg: &ObjectIFrameMsg,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] peer: EndpointId,
) -> anyhow::Result<()> {
    let mut buf = [0u8; OBJECT_IFRAME_MSG_MAX_SIZE];
    let bytes = postcard::to_slice(msg, &mut buf)?;

    let len = u32::try_from(bytes.len())?;
    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(bytes).await?;

    #[cfg(feature = "devtools-network")]
    {
        use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};
        let _ = NETWORK_EVENTS.0.try_send(NetworkEvent::Upload {
            peer,
            bytes: bytes.len() + 4,
            is_iframe: true,
        });
    }

    debug!(id = msg.id, "sent object I-frame");

    Ok(())
}

/// Send an object P-frame as a datagram.
/// Object ID is included in the datagram for routing (datagrams are shared).
pub fn send_object_pframe(
    connection: &Connection,
    msg: &ObjectPFrameDatagram,
    #[cfg_attr(not(feature = "devtools-network"), expect(unused))] peer: EndpointId,
) -> anyhow::Result<()> {
    let mut buf = [0u8; DATAGRAM_MAX_SIZE];
    let tagged = Datagram::ObjectPFrame(msg.clone());
    let bytes = postcard::to_slice(&tagged, &mut buf)?;

    if connection.datagram_send_buffer_space() < bytes.len() {
        debug!("datagram buffer full, dropping object P-frame");
        return Ok(());
    }

    connection.send_datagram(Bytes::copy_from_slice(bytes))?;

    #[cfg(feature = "devtools-network")]
    {
        use crate::devtools::events::{NETWORK_EVENTS, NetworkEvent};
        let _ = NETWORK_EVENTS.0.try_send(NetworkEvent::Upload {
            peer,
            bytes: bytes.len(),
            is_iframe: false,
        });
    }

    Ok(())
}
