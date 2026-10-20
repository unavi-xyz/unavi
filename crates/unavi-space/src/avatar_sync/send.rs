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
use unavi_avatar::{
    Grounded,
    bones::AvatarBones,
};

use crate::{
    avatar_sync::{
        AgentSender,
        OutgoingPose,
        RemoteAgent,
        crossing::LastCrossing,
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

/// Desktop tracks nothing but the look pitch, which the animation system does
/// not touch; the limbs are reproduced by the receiver's own locomotion
/// animation.
const DESKTOP_BONES: [BoneName; 1] = [BoneName::Head];

/// The bones this agent streams: its whole tracked pose in VR, just the head on
/// desktop.
const fn streamed_bones(is_xr: bool) -> &'static [BoneName] {
    if is_xr {
        &NETWORKED_BONES
    } else {
        &DESKTOP_BONES
    }
}

pub fn send_agent_pose(
    time: Res<Time>,
    active: Res<ActiveSpace>,
    last_crossing: Res<LastCrossing>,
    xr: Option<Res<InputMode>>,
    spaces: Query<&Space>,
    agent: Query<(&AgentAvatar, &LocalAgentEntities), With<LocalAgent>>,
    avatars: Query<&AvatarBones>,
    globals: Query<&GlobalTransform>,
    locals: Query<&Transform>,
    grounded: Query<&Grounded>,
    mut streams: Query<(Entity, &AgentSender, &SendInterval, &mut LastSent)>,
    commands: ParallelCommands,
) {
    let Ok((avatar, entities)) = agent.single() else {
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

    // A sender's airborne state is the one input to the avatar's animation
    // that no transform carries; peers replay the jump from it. Missing state
    // reads as standing, so a peer is never stuck falling.
    let is_grounded = grounded
        .get(entities.body)
        .map_or(true, |grounded| grounded.0);
    let is_xr = xr.is_some_and(|mode| mode.is_xr());
    let bones = avatars
        .get(avatar.0)
        .map_or_default(|bones| gather_bones(bones, &locals, streamed_bones(is_xr)));

    let outgoing = OutgoingPose {
        space:    space.id(),
        pose:     Pose {
            root: (&root).into(),
            bones,
        },
        grounded: is_grounded,
        crossing: last_crossing.latest(),
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
    names: &[BoneName],
) -> HashMap<BoneName, RigidTransform<F16Vec3>> {
    names
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
