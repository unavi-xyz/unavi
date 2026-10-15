//! Avatar sync: streaming the local agent's pose to each peer, and driving a
//! remote avatar from each peer's stream.

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use iroh::EndpointId;
use serde_vrm::vrm0::BoneName;

use crate::{
    index::{
        self,
        Indexed,
    },
    link::codec::{
        self,
        Keyframe,
        pose::Pose,
    },
    membership::SpaceId,
};

pub mod receive;
pub mod send;
pub mod wire;

/// A pose received from a peer, at full precision, in its space's frame.
pub struct ResolvedPose {
    pub space: SpaceId,
    pub root:  Transform,
    pub bones: HashMap<BoneName, Transform>,
}

impl ResolvedPose {
    /// `None` unless every value is finite and inside the space's cell.
    fn checked(mut self) -> Option<Self> {
        self.root.translation = codec::position(self.root.translation)?;
        self.root.rotation = codec::rotation(self.root.rotation)?;
        for bone in self.bones.values_mut() {
            bone.translation = codec::position(bone.translation)?;
            bone.rotation = codec::rotation(bone.rotation)?;
        }
        Some(self)
    }
}

/// The avatar of a connected peer, despawned with its connection.
#[derive(Component)]
#[component(
    immutable,
    on_insert = index::insert::<Self>,
    on_discard = index::discard::<Self>
)]
pub struct RemoteAgent(pub EndpointId);

impl Indexed for RemoteAgent {
    type Key = EndpointId;

    fn key(&self) -> EndpointId {
        self.0
    }
}

/// The local agent's pose for one tick, sent to every peer.
#[derive(Clone)]
pub struct OutgoingPose {
    pub space: SpaceId,
    pub pose:  Pose<Keyframe>,
}

/// Feeds one peer's avatar stream.
#[derive(Component)]
pub struct AgentSender(pub async_channel::Sender<OutgoingPose>);
