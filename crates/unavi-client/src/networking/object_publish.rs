//! Detect dynamic HSD objects, claim ownership, and publish
//! physics state to the network thread.

use std::{collections::HashMap, time::Duration};

use avian3d::dynamics::rigid_body::{AngularVelocity, LinearVelocity};
use avian3d::prelude::RigidBody;
use bevy::prelude::*;
use bevy_hsd::{NodeIndex, StageNode};

use crate::{
    networking::{
        publish_utils::{IFRAME_FREQ, PUBLISH_INTERVAL, transform_changed, velocity_changed},
        thread::{
            NetworkCommand, NetworkingThread,
            space::types::{
                f16_pos::F16Pos,
                object_id::ObjectId,
                physics_state::{PhysicsBaseline, PhysicsIFrame, PhysicsPFrame},
                velocity::F16Vel,
            },
        },
    },
    space::Space,
};

/// Marks a [`StageNode`] as a tracked dynamic object.
#[derive(Component)]
pub struct DynamicObject {
    pub object_id: ObjectId,
    pub claimed: bool,
}

#[derive(Component)]
pub struct Grabbed;

/// Detect newly-compiled `StageNode` entities with [`RigidBody::Dynamic`].
pub fn detect_dynamic_objects(
    mut commands: Commands,
    new_bodies: Query<
        (Entity, &StageNode, &NodeIndex, &RigidBody),
        (Added<RigidBody>, Without<DynamicObject>),
    >,
    spaces: Query<&Space>,
) {
    for (entity, stage_node, node_index, rigid_body) in &new_bodies {
        if *rigid_body != RigidBody::Dynamic {
            continue;
        }

        let Ok(space) = spaces.get(stage_node.stage) else {
            warn!(%entity, "dynamic object has no space");
            continue;
        };

        let object_id = ObjectId::new(space.0, node_index.0);

        commands.entity(entity).insert(DynamicObject {
            object_id,
            claimed: false,
        });
    }
}

/// Release ownership when a dynamic object loses its rigid body or
/// changes from dynamic.
pub fn detect_removed_objects(
    mut commands: Commands,
    nt: Res<NetworkingThread>,
    objects: Query<(Entity, &DynamicObject, Option<&RigidBody>)>,
) {
    for (entity, dyn_obj, rigid_body) in &objects {
        let dominated = rigid_body.is_none_or(|rb| *rb != RigidBody::Dynamic);
        if !dominated {
            continue;
        }

        if let Err(err) = nt
            .command_tx
            .try_send(NetworkCommand::ReleaseObject(dyn_obj.object_id))
        {
            error!(?err, "failed to send release");
        }

        commands.entity(entity).remove::<DynamicObject>();
    }
}

/// Per-object baseline state for P-frame delta encoding.
#[derive(Default)]
pub struct ObjectBaselines(HashMap<ObjectId, PhysicsBaseline>);

/// Publish physics state for owned dynamic objects.
pub fn publish_object_physics(
    nt: Res<NetworkingThread>,
    objects: Query<(
        &DynamicObject,
        &Transform,
        Option<&LinearVelocity>,
        Option<&AngularVelocity>,
    )>,
    time: Res<Time>,
    mut last: Local<Duration>,
    mut count: Local<u64>,
    mut baselines: Local<ObjectBaselines>,
) {
    let now = time.elapsed();
    if now.saturating_sub(*last) < PUBLISH_INTERVAL {
        return;
    }
    *last = now;
    *count += 1;

    let is_iframe = (*count).is_multiple_of(IFRAME_FREQ);

    if is_iframe {
        let mut frames = Vec::new();

        for (dyn_obj, transform, lin_vel, ang_vel) in &objects {
            if !dyn_obj.claimed {
                continue;
            }

            let pos = transform.translation;
            let rot = transform.rotation;
            let vel = lin_vel.map_or(Vec3::ZERO, |v| v.0);
            let avel = ang_vel.map_or(Vec3::ZERO, |v| v.0);

            baselines.0.insert(
                dyn_obj.object_id,
                PhysicsBaseline {
                    pos,
                    vel,
                    ang_vel: avel,
                },
            );

            frames.push((
                dyn_obj.object_id,
                PhysicsIFrame {
                    pos: pos.into(),
                    rot: rot.into(),
                    vel: vel.into(),
                    ang_vel: avel.into(),
                },
            ));
        }

        if !frames.is_empty()
            && let Err(err) = nt
                .command_tx
                .try_send(NetworkCommand::PublishObjectIFrame(frames))
        {
            error!(?err, "send error");
        }
    } else {
        let mut frames = Vec::new();

        for (dyn_obj, transform, lin_vel, ang_vel) in &objects {
            if !dyn_obj.claimed {
                continue;
            }

            let pos = transform.translation;
            let rot = transform.rotation;
            let vel = lin_vel.map_or(Vec3::ZERO, |v| v.0);
            let avel = ang_vel.map_or(Vec3::ZERO, |v| v.0);

            let baseline = baselines.0.get(&dyn_obj.object_id);
            let (base_pos, base_vel, base_ang_vel) =
                baseline.map_or((pos, vel, avel), |b| (b.pos, b.vel, b.ang_vel));

            // Epsilon filter: skip unchanged objects.
            if !transform_changed(pos, rot, base_pos, rot)
                && !velocity_changed(vel, base_vel)
                && !velocity_changed(avel, base_ang_vel)
            {
                continue;
            }

            frames.push((
                dyn_obj.object_id,
                PhysicsPFrame {
                    pos: F16Pos::from_delta(pos, base_pos),
                    rot: rot.into(),
                    vel: F16Vel::from_delta(vel, base_vel),
                    ang_vel: F16Vel::from_delta(avel, base_ang_vel),
                },
            ));
        }

        if !frames.is_empty()
            && let Err(err) = nt
                .command_tx
                .try_send(NetworkCommand::PublishObjectPFrame(frames))
        {
            error!(?err, "send error");
        }
    }
}
