//! Wire message types for space protocol.

use serde::{Deserialize, Serialize};

use super::types::{
    object_id::ObjectId,
    physics_state::{PhysicsIFrame, PhysicsPFrame},
    pose::{AgentIFrame, AgentPFrame},
};

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

// Object messages.

/// Single object's pose in a frame message.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectPose<T> {
    pub id: ObjectId,
    pub state: T,
}

/// Object I-frame message with full physics state.
///
/// Sent over reliable stream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectIFrameMsg {
    /// Monotonic counter (wraps at `u16::MAX`).
    pub id: u16,
    /// Timestamp for snapshot interpolation (millis since epoch).
    pub timestamp: u64,
    /// Objects with their full physics state.
    pub objects: Vec<ObjectPose<PhysicsIFrame>>,
}

/// Object P-frame datagram with delta physics state.
///
/// Sent unreliably.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectPFrameDatagram {
    /// Must match the latest I-frame ID to be accepted.
    pub iframe_id: u16,
    /// Monotonic sequence number within the current I-frame window.
    pub seq: u16,
    /// Timestamp for snapshot interpolation (millis since epoch).
    pub timestamp: u64,
    /// Objects with delta physics state (only changed objects included).
    pub objects: Vec<ObjectPose<PhysicsPFrame>>,
}

/// Tagged datagram for multiplexing agent and object P-frames.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Datagram {
    AgentPFrame(PFrameDatagram),
    ObjectPFrame(ObjectPFrameDatagram),
}
