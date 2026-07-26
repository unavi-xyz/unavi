use std::time::Duration;

use async_channel::TrySendError;
use bevy::{
    ecs::system::ParallelCommands,
    platform::collections::HashMap,
    prelude::*,
};
use blake3::Hash;
use serde_vrm::vrm0::BoneName;
use unavi_agent::{
    AgentAvatar,
    LocalAgent,
    config::XrMode,
};
use unavi_avatar::bones::AvatarBones;

use crate::{
    Space,
    anchor::ActiveSpace,
    connection::{
        ecs::{
            LastTick,
            PeerStream,
            Tickrate,
        },
        types::{
            IFrame,
            f16_vec3::F16Vec3,
            pose::{
                MAX_POSE_BONES,
                Pose,
            },
            rigid_transform::RigidTransform,
        },
    },
    peer::Peer,
};

/// Humanoid bones streamed to peers, ordered by visual significance and capped
/// at [`crate::connection::types::pose::MAX_POSE_BONES`]. Bones absent from an
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

#[derive(Clone)]
pub struct OutgoingPose {
    pub space: Hash,
    pub pose:  Pose<IFrame>,
}

#[derive(Component)]
#[require(Tickrate)]
pub struct AgentSender(pub async_channel::Sender<OutgoingPose>);

pub fn send_agent_pose(
    time: Res<Time>,
    active: Res<ActiveSpace>,
    xr: Option<Res<XrMode>>,
    spaces: Query<&Space>,
    agent: Query<&AgentAvatar, With<LocalAgent>>,
    avatars: Query<&AvatarBones>,
    globals: Query<&GlobalTransform>,
    locals: Query<&Transform>,
    mut streams: Query<(Entity, &AgentSender, &Tickrate, &mut LastTick)>,
    commands: ParallelCommands,
) {
    let Ok(avatar) = agent.single() else {
        return;
    };

    // The active space is the agent's space and sits at the world origin, so the
    // avatar's world transform is also its pose in that space's local frame.
    // Movement is driven by the body rigid-body rather than the `LocalAgent`
    // entity, hence reading the avatar's global transform.
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
    let bones = if xr.is_some_and(|xr| xr.0) {
        avatars
            .get(avatar.0)
            .map(|bones| gather_bones(bones, &locals))
            .unwrap_or_default()
    } else {
        HashMap::default()
    };

    let outgoing = OutgoingPose {
        space: space.0,
        pose:  Pose {
            root: (&root).into(),
            bones,
        },
    };

    let now = time.elapsed();

    streams
        .par_iter_mut()
        .for_each(|(entity, sender, tickrate, mut last_tick)| {
            if sender.0.is_full() {
                if sender.0.is_closed() {
                    commands.command_scope(|mut commands| commands.entity(entity).despawn());
                }
                return;
            }

            if last_tick.0 + tickrate.0 > now {
                return;
            }

            last_tick.0 = now;

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

const MAX_TICKRATE: Duration = Duration::from_millis(200);
const MIN_TICKRATE: Duration = Duration::from_millis(50);

const MAX_DIST: f32 = 50.0;
const MIN_DIST: f32 = 4.0;

pub fn set_agent_tickrates(
    agent: Query<&Transform, With<LocalAgent>>,
    peers: Query<(&Peer, &Transform), Without<LocalAgent>>,
    streams: Query<(&PeerStream, &mut Tickrate)>,
) {
    let Ok(root) = agent.single() else {
        return;
    };

    // Measure distance to each peer.
    for (target, mut tickrate) in streams {
        let Some((_, transform)) = peers.iter().find(|(p, _)| p.0.id == target.0) else {
            continue;
        };

        let dist = root.translation.distance(transform.translation);
        let dist = dist.abs().clamp(MIN_DIST, MAX_DIST);
        let s = (dist - MIN_DIST) / (MAX_DIST - MIN_DIST);
        let secs = MIN_TICKRATE
            .as_secs_f32()
            .lerp(MAX_TICKRATE.as_secs_f32(), s);
        tickrate.0 = Duration::from_secs_f32(secs);
    }
}
