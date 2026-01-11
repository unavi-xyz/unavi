//! Network monitoring events.

use std::sync::LazyLock;

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
pub static NETWORK_EVENTS: LazyLock<(flume::Sender<NetworkEvent>, flume::Receiver<NetworkEvent>)> =
    LazyLock::new(flume::unbounded);
