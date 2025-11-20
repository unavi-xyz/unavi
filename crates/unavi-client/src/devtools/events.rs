//! Network monitoring events.

use std::sync::LazyLock;

/// Network monitoring event types.
pub enum NetworkEvent {
    Download {
        host: String,
        bytes: usize,
    },
    Upload {
        host: String,
        bytes: usize,
        is_iframe: bool,
    },
    ValidTick {
        host: String,
    },
    DroppedFrame {
        host: String,
    },
}

/// Global channel for network monitoring events.
pub static NETWORK_EVENTS: LazyLock<(flume::Sender<NetworkEvent>, flume::Receiver<NetworkEvent>)> =
    LazyLock::new(flume::unbounded);
