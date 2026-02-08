//! Network monitoring events.

use std::sync::{LazyLock, Mutex};

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
pub static NETWORK_EVENTS: LazyLock<(
    tokio::sync::mpsc::Sender<NetworkEvent>,
    Mutex<tokio::sync::mpsc::Receiver<NetworkEvent>>,
)> = LazyLock::new(|| {
    let (tx, rx) = tokio::sync::mpsc::channel(64);
    (tx, Mutex::new(rx))
});
