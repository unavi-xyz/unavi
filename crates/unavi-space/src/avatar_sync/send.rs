//! Sending the local agent's pose to each peer, more often to nearer ones.

use std::time::Duration;

use async_channel::TrySendError;
use bevy::{
    ecs::system::ParallelCommands,
    platform::collections::HashMap,
    prelude::*,
};
use serde_vrm::vrm0::BoneName;
use unavi_agent::{
    AgentAvatar,
    LocalAgent,
    LocalAgentEntities,
    config::InputMode,
};
use unavi_avatar::bones::AvatarBones;

use crate::{
    avatar_sync::{
        AgentSender,
        OutgoingPose,
        RemoteAgent,
    },
    grid::ActiveSpace,
    index::Index,
    link::{
        codec::{
            f16_vec3::F16Vec3,
            pose::{
                MAX_POSE_BONES,
                Pose,
            },
            rigid_transform::RigidTransform,
        },
        senders::{
            LastSent,
            PeerSender,
            SendInterval,
        },
    },
    membership::Space,
};

/// Humanoid bones streamed to peers, ordered by visual significance and capped
/// at [`MAX_POSE_BONES`]. Bones absent from an
/// avatar's rig are simply skipped.
const NETWORKED_BONES: [BoneName; 11] = [
    BoneName::Hips,
    BoneName::Spine,
    BoneName::Chest,
    BoneName::Neck,
    BoneName::Head,
    BoneName::LeftUpperArm,
    BoneName::LeftLowerArm,
    BoneName::LeftHand,
    BoneName::RightUpperArm,
    BoneName::RightLowerArm,
    BoneName::RightHand,
];

const _: () = assert!(NETWORKED_BONES.len() <= MAX_POSE_BONES);

pub fn send_agent_pose(
    time: Res<Time>,
    active: Res<ActiveSpace>,
    xr: Option<Res<InputMode>>,
    spaces: Query<&Space>,
    agent: Query<&AgentAvatar, With<LocalAgent>>,
    avatars: Query<&AvatarBones>,
    globals: Query<&GlobalTransform>,
    locals: Query<&Transform>,
    mut streams: Query<(Entity, &AgentSender, &SendInterval, &mut LastSent)>,
    commands: ParallelCommands,
) {
    let Ok(avatar) = agent.single() else {
        return;
    };

    // The active space sits at the world origin, so the avatar's world
    // transform is its pose in the space's local frame. The body
    // rigid-body, not the `LocalAgent`, drives movement, hence the avatar's
    // global transform.
    let Some(root) = globals
        .get(avatar.0)
        .ok()
        .map(GlobalTransform::compute_transform)
    else {
        return;
    };

    let Some(space) = active.0.and_then(|e| spaces.get(e).ok()) else {
        return;
    };

    // Bone tracking is only meaningful in VR, where limbs are driven by real
    // pose data; on desktop peers reconstruct limbs from locomotion animation.
    let bones = if xr.is_some_and(|mode| mode.is_xr()) {
        avatars
            .get(avatar.0)
            .map_or_default(|bones| gather_bones(bones, &locals))
    } else {
        HashMap::default()
    };

    let outgoing = OutgoingPose {
        space: space.id(),
        pose:  Pose {
            root: (&root).into(),
            bones,
        },
    };

    let now = time.elapsed();

    streams
        .par_iter_mut()
        .for_each(|(entity, sender, interval, mut last_sent)| {
            if sender.0.is_full() {
                if sender.0.is_closed() {
                    commands.command_scope(|mut commands| commands.entity(entity).despawn());
                }
                return;
            }

            if !last_sent.take_due(interval.0, now) {
                return;
            }

            match sender.0.try_send(outgoing.clone()) {
                Ok(()) | Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Closed(_)) => {
                    commands.command_scope(|mut commands| commands.entity(entity).despawn());
                }
            }
        });
}

fn gather_bones(
    bones: &AvatarBones,
    locals: &Query<&Transform>,
) -> HashMap<BoneName, RigidTransform<F16Vec3>> {
    NETWORKED_BONES
        .iter()
        .filter_map(|name| {
            let entity = *bones.get(name)?;
            let transform = locals.get(entity).ok()?;
            Some((*name, transform.into()))
        })
        .collect()
}

const MAX_INTERVAL: Duration = Duration::from_millis(200);
const MIN_INTERVAL: Duration = Duration::from_millis(50);

const MAX_DIST: f32 = 50.0;
const MIN_DIST: f32 = 4.0;

/// Sends less often to peers whose avatar is far from the local body.
pub fn set_agent_intervals(
    agent: Query<&LocalAgentEntities, With<LocalAgent>>,
    globals: Query<&GlobalTransform>,
    agents: Res<Index<RemoteAgent>>,
    mut senders: Query<(&PeerSender, &mut SendInterval), With<AgentSender>>,
) {
    let Some(body) = agent
        .single()
        .ok()
        .and_then(|entities| globals.get(entities.body).ok())
        .map(GlobalTransform::translation)
    else {
        return;
    };

    for (peer, mut interval) in &mut senders {
        let Some(remote) = agents
            .get(peer.0)
            .and_then(|e| globals.get(e).ok())
            .map(GlobalTransform::translation)
        else {
            continue;
        };

        let dist = body.distance(remote).clamp(MIN_DIST, MAX_DIST);
        let s = (dist - MIN_DIST) / (MAX_DIST - MIN_DIST);
        let secs = MIN_INTERVAL
            .as_secs_f32()
            .lerp(MAX_INTERVAL.as_secs_f32(), s);
        interval.0 = Duration::from_secs_f32(secs);
    }
}
