//! `wired:agent`: the local user's camera and body.

use bevy::prelude::*;
use bevy_vrm::BoneName;
use hsd::attributes::xform::XformAttr;
use unavi_policy::permissions::HostApi;

use crate::{
    error::ScriptError,
    host::{
        ScriptHost,
        scene::place::{
            Anchor,
            place,
        },
        shared_state::agents::Tracked,
    },
};

#[derive(Clone, Copy, Debug)]
pub enum Attachment {
    Camera,
    Bone(BoneName),
}

fn world_transform(host: &ScriptHost, tracked: Tracked) -> Result<Transform, ScriptError> {
    let snapshot = host
        .shared
        .transforms
        .node(&tracked.node)
        .ok_or(ScriptError::NotReady)?;
    Ok(snapshot.world.compute_transform())
}

pub fn camera_transform(host: &ScriptHost) -> Result<Transform, ScriptError> {
    host.require(HostApi::LocalAgent)?;
    let camera = host.shared.agents.camera().ok_or(ScriptError::NotReady)?;
    world_transform(host, camera)
}

/// `None` when the avatar lacks `bone`.
pub fn bone_transform(host: &ScriptHost, bone: BoneName) -> Result<Option<Transform>, ScriptError> {
    host.require(HostApi::LocalAgent)?;
    if !host.shared.agents.is_ready() {
        return Err(ScriptError::NotReady);
    }
    host.shared
        .agents
        .bone(bone)
        .map(|tracked| world_transform(host, tracked))
        .transpose()
}

/// Places a document the script owns relative to the camera or a bone.
pub async fn attach(
    host: &ScriptHost,
    doc: u32,
    to: Attachment,
    offset: XformAttr,
) -> Result<(), ScriptError> {
    host.require(HostApi::LocalAgent)?;
    let tracked = match to {
        Attachment::Camera => host.shared.agents.camera(),
        Attachment::Bone(bone) => host.shared.agents.bone(bone),
    }
    .ok_or(ScriptError::NotReady)?;
    place(host, doc, Anchor::Entity(tracked.entity), offset).await
}
