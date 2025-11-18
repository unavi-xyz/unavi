//! Network monitoring events.

use std::sync::{LazyLock, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

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
pub static NETWORK_EVENTS: LazyLock<(
    UnboundedSender<NetworkEvent>,
    Mutex<UnboundedReceiver<NetworkEvent>>,
)> = LazyLock::new(|| {
    let (tx, rx) = unbounded_channel();
    (tx, Mutex::new(rx))
});
