//! Driving remote avatars from received poses, smoothed between updates.

use std::time::Duration;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use serde_vrm::vrm0::BoneName;
use unavi_avatar::{
    Avatar,
    Grounded,
    animation::{
        defaults::default_character_animations,
        velocity::AverageVelocity,
    },
    bones::AvatarBones,
};
use unavi_portal::EchoBody;
use web_time::Instant;

use crate::{
    avatar_sync::{
        RemoteAgent,
        ResolvedPose,
    },
    discovery::{
        LastSeenIn,
        Peer,
    },
    index::Index,
    link::PeerLink,
    membership::Space,
};

const MIN_LERP: Duration = Duration::from_millis(50);
const MAX_LERP: Duration = Duration::from_millis(500);

struct TransformLerp {
    prev:   Transform,
    target: Transform,
}

impl TransformLerp {
    const fn snapped(transform: Transform) -> Self {
        Self {
            prev:   transform,
            target: transform,
        }
    }

    fn sample(&self, t: f32) -> (Vec3, Quat) {
        (
            self.prev.translation.lerp(self.target.translation, t),
            self.prev.rotation.slerp(self.target.rotation, t),
        )
    }

    fn retarget(&mut self, t: f32, target: Transform) {
        let (translation, rotation) = self.sample(t);
        self.prev = Transform {
            translation,
            rotation,
            scale: self.prev.scale,
        };
        self.target = target;
    }
}

#[derive(Component)]
pub struct PoseLerp {
    root:      TransformLerp,
    bones:     HashMap<BoneName, TransformLerp>,
    elapsed:   Duration,
    duration:  Duration,
    last_recv: Instant,
}

impl PoseLerp {
    fn snapped(pose: &ResolvedPose, recv: Instant) -> Self {
        Self {
            root:      TransformLerp::snapped(pose.root),
            bones:     pose
                .bones
                .iter()
                .map(|(name, transform)| (*name, TransformLerp::snapped(*transform)))
                .collect(),
            elapsed:   Duration::ZERO,
            duration:  MIN_LERP,
            last_recv: recv,
        }
    }

    fn frac(&self) -> f32 {
        if self.duration.is_zero() {
            1.0
        } else {
            (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
        }
    }

    fn retarget(&mut self, pose: &ResolvedPose, recv: Instant) {
        let t = self.frac();
        self.root.retarget(t, pose.root);
        for (name, target) in &pose.bones {
            self.bones
                .entry(*name)
                .and_modify(|bone| bone.retarget(t, *target))
                .or_insert_with(|| TransformLerp::snapped(*target));
        }
        self.elapsed = Duration::ZERO;
        self.duration = recv
            .saturating_duration_since(self.last_recv)
            .clamp(MIN_LERP, MAX_LERP);
        self.last_recv = recv;
    }
}

/// Moves each peer's avatar toward its latest pose, spawning it on first
/// sight. A pose naming a space the peer is not present in is dropped.
pub fn apply_remote_poses(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    space_index: Res<Index<Space>>,
    peer_index: Res<Index<Peer>>,
    agent_index: Res<Index<RemoteAgent>>,
    mut peers: Query<&mut LastSeenIn, With<Peer>>,
    mut remotes: Query<(&ChildOf, &mut PoseLerp), With<RemoteAgent>>,
    link: Option<Res<PeerLink>>,
    mut commands: Commands,
) {
    let Some(link) = link else {
        return;
    };
    let now = time.elapsed();

    for (peer, (recv, resolved)) in link.poses().drain() {
        // Only a space the peer announced itself in; the pose stream then
        // keeps that presence fresh while it flows.
        let Some(mut seen) = peer_index.get(peer).and_then(|e| peers.get_mut(e).ok()) else {
            continue;
        };
        if !seen.contains(resolved.space) {
            continue;
        }
        seen.refresh(resolved.space, now);

        let agent = agent_index.get(peer);
        let Some(space) = space_index.get(resolved.space) else {
            // Space not instanced (may still be loading); drop any orphaned
            // avatar and wait for it.
            if let Some(agent) = agent {
                commands.entity(agent).despawn();
            }
            continue;
        };

        let Some((agent, (child_of, mut lerp))) =
            agent.and_then(|e| remotes.get_mut(e).ok().map(|r| (e, r)))
        else {
            info!(%peer, space = %resolved.space, "Instancing remote agent");
            let mut remote = commands.spawn((
                RemoteAgent(peer),
                Avatar,
                EchoBody,
                Grounded(true),
                default_character_animations(&asset_server),
                resolved.root,
                PoseLerp::snapped(&resolved, recv),
                ChildOf(space),
            ));
            let remote_id = remote.id();
            remote.insert(AverageVelocity {
                target: Some(remote_id),
                ..Default::default()
            });
            continue;
        };

        if child_of.parent() == space {
            lerp.retarget(&resolved, recv);
        } else {
            // Space changed: reparent and snap rather than lerp across the grid
            // jump.
            commands.entity(agent).insert(ChildOf(space));
            *lerp = PoseLerp::snapped(&resolved, recv);
        }
    }
}

pub fn advance_remote_lerp(time: Res<Time>, mut remotes: Query<(&mut PoseLerp, &mut Transform)>) {
    let dt = time.delta();

    for (mut lerp, mut transform) in &mut remotes {
        lerp.elapsed = (lerp.elapsed + dt).min(lerp.duration);
        let t = lerp.frac();

        let (translation, rotation) = lerp.root.sample(t);
        transform.translation = translation;
        transform.rotation = rotation;
    }
}

/// Overwrites animated bones with networked pose data. Runs after the
/// animation system so it wins over the locomotion animation for tracked bones,
/// leaving untracked bones fully animation-driven.
pub fn apply_remote_bones(
    remotes: Query<(&PoseLerp, &AvatarBones)>,
    mut bones: Query<&mut Transform>,
) {
    for (lerp, avatar_bones) in &remotes {
        let t = lerp.frac();

        for (name, bone_lerp) in &lerp.bones {
            let Some(&entity) = avatar_bones.get(name) else {
                continue;
            };
            let Ok(mut bone) = bones.get_mut(entity) else {
                continue;
            };
            let (translation, rotation) = bone_lerp.sample(t);
            bone.translation = translation;
            bone.rotation = rotation;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retarget_preserves_current_as_new_prev() {
        let mut lerp = TransformLerp {
            prev:   Transform::from_xyz(0.0, 0.0, 0.0),
            target: Transform::from_xyz(2.0, 0.0, 0.0),
        };
        lerp.retarget(0.5, Transform::from_xyz(4.0, 0.0, 0.0));
        assert!((lerp.prev.translation - Vec3::new(1.0, 0.0, 0.0)).length() < 0.001);
        assert!((lerp.target.translation - Vec3::new(4.0, 0.0, 0.0)).length() < 0.001);
    }
}
