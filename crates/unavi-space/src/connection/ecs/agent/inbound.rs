use std::time::Duration;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use iroh::EndpointId;
use iroh_docs::NamespaceId;
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
use unavi_manifold::EchoBody;
use unavi_policy::space::Space;
use web_time::Instant;

use crate::{
    connection::PeerLink,
    peer::{
        ActiveSpaces,
        Peer,
    },
};

const MIN_LERP: Duration = Duration::from_millis(50);
const MAX_LERP: Duration = Duration::from_millis(500);

pub struct ResolvedPose {
    pub space: NamespaceId,
    pub root:  Transform,
    pub bones: HashMap<BoneName, Transform>,
}

#[derive(Component)]
pub struct RemoteAgent(pub EndpointId);

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

pub fn apply_remote_poses(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    spaces: Query<(Entity, &Space)>,
    mut peers: Query<(&Peer, &mut ActiveSpaces)>,
    mut remotes: Query<(Entity, &RemoteAgent, &ChildOf, &mut PoseLerp)>,
    link: Option<Res<PeerLink>>,
    mut commands: Commands,
) {
    let Some(link) = link else {
        return;
    };
    let updates = link.poses().drain();
    let now = time.elapsed_secs();

    for (peer, (recv, resolved)) in updates {
        // Pose stream is the live presence signal; refresh so a connected peer
        // never expires from stale discovery gossip.
        if let Some((_, mut active_spaces)) = peers.iter_mut().find(|(p, _)| p.0.id == peer) {
            active_spaces.0.insert(resolved.space, now);
        }

        let Some((space, _)) = spaces.iter().find(|(_, s)| s.0 == resolved.space) else {
            // Space not instanced (may still be loading); drop any orphaned
            // avatar and wait for it.
            if let Some((entity, ..)) = remotes.iter().find(|(_, r, ..)| r.0 == peer) {
                commands.entity(entity).despawn();
            }
            continue;
        };

        let Some((entity, _, child_of, mut lerp)) =
            remotes.iter_mut().find(|(_, r, ..)| r.0 == peer)
        else {
            info!(peer = %peer, space = %resolved.space, pos = ?resolved.root.translation, "Instancing remote agent");
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
            commands.entity(entity).insert(ChildOf(space));
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

pub fn despawn_remote_agent(
    trigger: On<Remove, Peer>,
    peers: Query<&Peer>,
    remotes: Query<(Entity, &RemoteAgent)>,
    mut commands: Commands,
) {
    let Ok(peer) = peers.get(trigger.entity) else {
        return;
    };

    for (entity, remote) in remotes {
        if remote.0 == peer.0.id {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_lerp_samples_midpoint() {
        let lerp = TransformLerp {
            prev:   Transform::from_xyz(0.0, 0.0, 0.0),
            target: Transform::from_xyz(2.0, 0.0, 0.0),
        };
        let (translation, _) = lerp.sample(0.5);
        assert!((translation - Vec3::new(1.0, 0.0, 0.0)).length() < 0.001);
    }

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
