//! Wire message types for space protocol.

use serde::{Deserialize, Serialize};

use super::pose::{PlayerIFrame, PlayerPFrame};

/// I-frame message sent over reliable stream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IFrameMsg {
    /// Monotonic counter (wraps at `u32::MAX`).
    pub id: u32,
    pub pose: PlayerIFrame,
}

/// P-frame datagram sent unreliably.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PFrameDatagram {
    /// Must match the latest I-frame ID to be accepted.
    pub iframe_id: u32,
    pub pose: PlayerPFrame,
}

/// Control stream messages for tickrate negotiation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ControlMsg {
    /// Request a tickrate. Sender proposes their desired Hz.
    TickrateRequest { hz: u8 },
    /// Acknowledge tickrate. Receiver responds with effective Hz.
    TickrateAck { hz: u8 },
}
