//! Network monitoring events.

use std::sync::{LazyLock, Mutex};

use iroh::EndpointId;

/// Network monitoring event types.
pub enum NetworkEvent {
    Download {
        peer: EndpointId,
        bytes: usize,
    },
    Upload {
        peer: EndpointId,
        bytes: usize,
        is_iframe: bool,
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
