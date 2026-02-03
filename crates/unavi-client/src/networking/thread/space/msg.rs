//! Wire message types for space protocol.

use serde::{Deserialize, Serialize};

use super::types::{AgentIFrame, AgentPFrame};

/// I-frame message sent over reliable stream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IFrameMsg {
    /// Monotonic counter (wraps at `u16::MAX`).
    pub id: u16,
    pub pose: AgentIFrame,
}

/// P-frame datagram sent unreliably.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PFrameDatagram {
    /// Must match the latest I-frame ID to be accepted.
    pub iframe_id: u16,
    /// Monotonic sequence number within the current I-frame window.
    /// Resets to 1 when a new I-frame is sent.
    pub seq: u16,
    pub pose: AgentPFrame,
}

/// Control stream messages for tickrate negotiation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ControlMsg {
    /// Request a tickrate. Sender proposes their desired Hz.
    TickrateRequest { hz: u8 },
    /// Acknowledge tickrate. Receiver responds with effective Hz.
    TickrateAck { hz: u8 },
}
