use std::{
    sync::{
        LazyLock,
        Mutex,
    },
    time::{
        Duration,
        Instant,
    },
};

use async_channel::TrySendError;
use avian3d::prelude::{
    AngularVelocity,
    LinearVelocity,
    Position,
    RigidBody,
    Rotation,
};
use bevy::{
    ecs::system::ParallelCommands,
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::{
    HsdChild,
    HsdPrimIndex,
    HsdRecordId,
    Prim,
};
use blake3::Hash;
use iroh::EndpointId;
use loro::TreeID;

use crate::{
    Space,
    membership,
    state::replicas,
};

/// One owned dynamic prim's space-relative pose, queued for broadcast. Captured
/// once per tick and cloned to each [`ObjectSender`].
#[derive(Clone)]
pub struct OutgoingObject {
    pub doc:   Hash,
    pub space: Hash,
    pub prim:  TreeID,
    pub root:  Transform,
    pub lin:   Vec3,
    pub ang:   Vec3,
}

#[derive(Component)]
#[require(LastObjectTick)]
pub struct ObjectSender(pub async_channel::Sender<Vec<OutgoingObject>>);

#[derive(Component, Default)]
pub struct LastObjectTick(Duration);

/// Objects move slowly relative to avatars and are dead-reckoned between
/// updates, so they tick far less often than agent poses.
const OBJECT_TICKRATE: Duration = Duration::from_millis(200);

/// Broadcasts every dynamic prim in documents the local peer has authority
/// over. Poses are space-relative; velocities are space-invariant (spaces only
/// translate).
pub fn send_object_poses(
    time: Res<Time>,
    spaces: Query<(&Space, &GlobalTransform)>,
    roots: Query<&HsdRecordId>,
    prims: Query<(
        &Prim,
        &HsdChild,
        &RigidBody,
        &GlobalTransform,
        Option<&LinearVelocity>,
        Option<&AngularVelocity>,
    )>,
    mut streams: Query<(Entity, &ObjectSender, &mut LastObjectTick)>,
    commands: ParallelCommands,
) {
    let now = time.elapsed();

    let space_origins = spaces
        .iter()
        .map(|(space, gt)| (space.0, gt.translation()))
        .collect::<HashMap<_, _>>();

    let outgoing = prims
        .iter()
        .filter_map(|(prim, child_of, body, global, lin, ang)| {
            if !matches!(body, RigidBody::Dynamic) {
                return None;
            }
            let doc = roots.get(child_of.0).ok()?.0;
            let space = membership::doc_space(doc)?;
            if !replicas::is_self_authority(space, doc) {
                return None;
            }
            let origin = space_origins.get(&space)?;
            let world = global.compute_transform();
            Some(OutgoingObject {
                doc,
                space,
                prim: prim.0,
                root: Transform {
                    translation: world.translation - *origin,
                    rotation: world.rotation,
                    ..Default::default()
                },
                lin: lin.map_or(Vec3::ZERO, |v| v.0),
                ang: ang.map_or(Vec3::ZERO, |v| v.0),
            })
        })
        .collect::<Vec<_>>();

    if outgoing.is_empty() {
        return;
    }

    streams
        .par_iter_mut()
        .for_each(|(entity, sender, mut last_tick)| {
            if sender.0.is_full() {
                if sender.0.is_closed() {
                    commands.command_scope(|mut commands| commands.entity(entity).despawn());
                }
                return;
            }
            if last_tick.0 + OBJECT_TICKRATE > now {
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

pub struct ResolvedObject {
    pub doc:   Hash,
    pub space: Hash,
    pub prim:  TreeID,
    pub root:  Transform,
    pub lin:   Vec3,
    pub ang:   Vec3,
}

/// Latest update received per `(peer, doc, prim)`, so a peer can drive every
/// prim of every document it owns independently.
static OBJECT_INBOX: LazyLock<
    Mutex<HashMap<(EndpointId, Hash, TreeID), (Instant, ResolvedObject)>>,
> = LazyLock::new(|| Mutex::new(HashMap::default()));

pub fn submit_object(peer: EndpointId, resolved: ResolvedObject) {
    OBJECT_INBOX.lock().expect("object inbox").insert(
        (peer, resolved.doc, resolved.prim),
        (Instant::now(), resolved),
    );
}

/// Drives a remotely-owned prim from network updates. The replica is held
/// [`RigidBody::Kinematic`] so local physics never fights the owner; its pose
/// is smoothed toward the last received space-relative target each tick.
#[derive(Component)]
pub struct ObjectInterp {
    space:     Entity,
    target:    Transform,
    lin:       Vec3,
    ang:       Vec3,
    last_recv: Instant,
}

/// Window a target is dead-reckoned past its last update before holding.
const MAX_EXTRAPOLATION: Duration = Duration::from_millis(300);

/// Exponential rate the replica chases its target; converges in ~100ms.
const SMOOTH_RATE: f32 = 16.0;

pub fn apply_remote_objects(
    roots: Query<(&HsdRecordId, &HsdPrimIndex)>,
    spaces: Query<(Entity, &Space)>,
    mut interps: Query<&mut ObjectInterp>,
    mut commands: Commands,
) {
    let updates = std::mem::take(&mut *OBJECT_INBOX.lock().expect("object inbox"));

    for ((peer, doc, prim), (recv, resolved)) in updates {
        // Only the document's current authority may move it, and never ourselves.
        if replicas::authority(resolved.space, doc) != Some(*peer.as_bytes())
            || replicas::is_self_authority(resolved.space, doc)
        {
            continue;
        }
        let Some(prim_entity) = roots
            .iter()
            .find(|(record, _)| record.0 == doc)
            .and_then(|(_, index)| index.0.get(&prim).copied())
        else {
            continue;
        };
        let Some((space, _)) = spaces.iter().find(|(_, s)| s.0 == resolved.space) else {
            continue;
        };

        if let Ok(mut interp) = interps.get_mut(prim_entity) {
            interp.space = space;
            interp.target = resolved.root;
            interp.lin = resolved.lin;
            interp.ang = resolved.ang;
            interp.last_recv = recv;
        } else {
            commands.entity(prim_entity).insert((
                ObjectInterp {
                    space,
                    target: resolved.root,
                    lin: resolved.lin,
                    ang: resolved.ang,
                    last_recv: recv,
                },
                RigidBody::Kinematic,
                ReplicaObject,
            ));
        }
    }
}

/// Smooths each replica toward its velocity-extrapolated target. The body is
/// kinematic, so writing [`Position`]/[`Rotation`] places it without
/// contention.
pub fn advance_object_interp(
    time: Res<Time>,
    spaces: Query<&GlobalTransform, With<Space>>,
    mut objects: Query<(&ObjectInterp, &mut Position, &mut Rotation)>,
) {
    let now = Instant::now();
    let alpha = 1.0 - (-SMOOTH_RATE * time.delta_secs()).exp();

    for (interp, mut position, mut rotation) in &mut objects {
        let Ok(origin) = spaces.get(interp.space).map(GlobalTransform::translation) else {
            continue;
        };

        let extrap = now
            .saturating_duration_since(interp.last_recv)
            .min(MAX_EXTRAPOLATION)
            .as_secs_f32();
        let target_pos = origin
            + interp
                .lin
                .mul_add(Vec3::splat(extrap), interp.target.translation);
        let target_rot = Quat::from_scaled_axis(interp.ang * extrap) * interp.target.rotation;

        position.0 = position.0.lerp(target_pos, alpha);
        rotation.0 = rotation.0.slerp(target_rot, alpha);
    }
}

/// A prim parked [`RigidBody::Kinematic`] because a remote peer has authority
/// over its document, so a fresh replica never free-falls before its first
/// update.
#[derive(Component)]
pub struct ReplicaObject;

/// Parks remotely-controlled prims as kinematic replicas and runs
/// ours/unclaimed ones as dynamic. `replicas::authority` resolves the latest
/// claim, so only the accepted controller drives a prim.
pub fn reconcile_object_authority(
    roots: Query<&HsdRecordId>,
    prims: Query<(Entity, &HsdChild, &RigidBody, Has<ReplicaObject>), With<Prim>>,
    mut commands: Commands,
) {
    for (entity, child_of, body, is_replica) in &prims {
        let Some(doc) = roots.get(child_of.0).ok().map(|r| r.0) else {
            continue;
        };
        let remote_controlled = membership::doc_space(doc).is_some_and(|space| {
            replicas::authority(space, doc).is_some() && !replicas::is_self_authority(space, doc)
        });

        match (remote_controlled, is_replica) {
            (true, false) if matches!(body, RigidBody::Dynamic) => {
                commands
                    .entity(entity)
                    .insert((ReplicaObject, RigidBody::Kinematic));
            }
            (false, true) => {
                commands
                    .entity(entity)
                    .remove::<(ReplicaObject, ObjectInterp)>()
                    .insert(RigidBody::Dynamic);
            }
            _ => {}
        }
    }
}
