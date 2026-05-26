//! Network monitoring events.

use std::sync::LazyLock;

use async_channel::{Receiver, Sender};
use iroh::EndpointId;

/// Channel type for bandwidth tracking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    Datagram,
    Stream,
}

/// Network monitoring event types.
pub enum NetworkEvent {
    Download {
        peer: EndpointId,
        bytes: usize,
        channel: Channel,
    },
    Upload {
        peer: EndpointId,
        bytes: usize,
        channel: Channel,
    },
    ValidTick {
        peer: EndpointId,
    },
    DroppedFrame {
        peer: EndpointId,
    },
}

/// Global channel for network monitoring events.
pub static NETWORK_EVENTS: LazyLock<(Sender<NetworkEvent>, Receiver<NetworkEvent>)> =
    LazyLock::new(|| async_channel::bounded(64));
