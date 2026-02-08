//! Wire message types for space protocol.

use serde::{Deserialize, Serialize};

use super::types::{
    object_id::ObjectId,
    physics_state::{PhysicsIFrame, PhysicsPFrame},
    pose::{AgentIFrame, AgentPFrame},
};

/// Identifies a stream's purpose on first message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StreamInit {
    AgentControl,
    AgentIFrame,
    Object { object_id: ObjectId },
}

/// I-frame message sent over reliable stream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentIFrameMsg {
    /// Monotonic counter (wraps at `u16::MAX`).
    pub id: u16,
    pub pose: AgentIFrame,
}

/// P-frame datagram sent unreliably.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentPFrameDatagram {
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

// Object messages.

/// Object I-frame message for a single object (sent on its dedicated stream).
/// Object ID is known from `StreamInit::Object` sent at stream creation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectIFrameMsg {
    /// Monotonic counter (wraps at `u16::MAX`).
    pub id: u16,
    /// Timestamp for snapshot interpolation (millis since epoch).
    pub timestamp: u64,
    /// Full physics state.
    pub state: PhysicsIFrame,
}

/// Object P-frame datagram (unreliable, shared channel).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectPFrameDatagram {
    /// Must match the latest I-frame ID to be accepted.
    pub iframe_id: u16,
    /// Monotonic sequence number within the current I-frame window.
    pub seq: u16,
    /// Timestamp for snapshot interpolation (millis since epoch).
    pub timestamp: u64,
    /// Tag for routing to correct object handler.
    pub object_id: ObjectId,
    /// Delta physics state.
    pub state: PhysicsPFrame,
}

/// Tagged datagram for multiplexing agent and object P-frames.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Datagram {
    AgentPFrame(AgentPFrameDatagram),
    ObjectPFrame(ObjectPFrameDatagram),
}
