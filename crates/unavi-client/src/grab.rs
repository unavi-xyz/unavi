use std::{collections::HashMap, time::Duration};

use avian3d::prelude::{AngularVelocity, GravityScale, LinearVelocity, RayHits};
use bevy::prelude::*;
use unavi_input::{SqueezeDown, SqueezeUp, crosshair::CrosshairMode, raycast::PrimaryRaycastInput};

use crate::networking::{
    object_publish::{DynObjectId, Grabbed, LocallyOwned},
    thread::{
        NetworkCommand, NetworkingThread,
        space::{object::outbound::LocalGrabbedObjects, types::object_id::ObjectId},
    },
};

const GRAB_COOLDOWN: Duration = Duration::from_millis(100);
const GRAB_DEAD_ZONE: f32 = 0.001;
const GRAB_ROTATION_DEAD_ZONE: f32 = 0.01;
const GRAB_SMOOTHING: f32 = 10.0;

#[derive(Resource, Default)]
pub struct GrabbedObjects(HashMap<Entity, GrabState>);

struct GrabState {
    entity: Entity,
    object_id: ObjectId,
    position_offset: Vec3,
    rotation_offset: Quat,
}

pub fn handle_squeeze_up(
    event: On<SqueezeUp>,
    mut commands: Commands,
    mut grabbed: ResMut<GrabbedObjects>,
    mut local_grabbed: ResMut<LocalGrabbedObjects>,
    nt: Res<NetworkingThread>,
) {
    // Drop the currently grabbed object (if any).
    if let Some(prev) = grabbed.0.remove(&event.pointer) {
        commands.entity(prev.entity).remove::<Grabbed>();
        local_grabbed.0.remove(&prev.object_id);

        // Send updated grabbed objects to network thread.
        let _ = nt.command_tx.try_send(NetworkCommand::UpdateGrabbedObjects(
            local_grabbed.0.clone(),
        ));
    }
}

pub fn handle_squeeze_down(
    event: On<SqueezeDown>,
    mut commands: Commands,
    mut grabbed: ResMut<GrabbedObjects>,
    mut local_grabbed: ResMut<LocalGrabbedObjects>,
    time: Res<Time>,
    nt: Res<NetworkingThread>,
    dyn_objs: Query<&DynObjectId, Without<Grabbed>>,
    locally_owned: Query<(), With<LocallyOwned>>,
    transforms: Query<&GlobalTransform>,
    mut last_grab: Local<Duration>,
) {
    let now = time.elapsed();
    if now.saturating_sub(*last_grab) < GRAB_COOLDOWN {
        return;
    }

    let target_ent = event.event().entity;
    let Ok(obj) = dyn_objs.get(target_ent) else {
        // Not a dynamic object or already grabbed
        return;
    };

    let Ok(pointer_transform) = transforms.get(event.pointer) else {
        warn!(pointer = %event.pointer, "pointer transform not found");
        return;
    };

    let Ok(obj_transform) = transforms.get(target_ent) else {
        warn!(entity = %target_ent, "object transform not found");
        return;
    };

    info!(pointer = %event.pointer, object = %obj.0.index, "grabbing object");

    let pointer_tr = pointer_transform.compute_transform();
    let obj_tr = obj_transform.compute_transform();

    // Store offsets in pointer's local space so they stay correct when rotating.
    let position_offset =
        pointer_tr.rotation.inverse() * (obj_tr.translation - pointer_tr.translation);
    let rotation_offset = pointer_tr.rotation.inverse() * obj_tr.rotation;

    grabbed.0.insert(
        event.pointer,
        GrabState {
            entity: target_ent,
            object_id: obj.0,
            position_offset,
            rotation_offset,
        },
    );
    *last_grab = now;

    commands.entity(target_ent).insert(Grabbed);
    local_grabbed.0.insert(obj.0);

    // Send updated grabbed objects to network thread.
    let _ = nt.command_tx.try_send(NetworkCommand::UpdateGrabbedObjects(
        local_grabbed.0.clone(),
    ));

    // Only send claim if not already locally owned.
    if !locally_owned.contains(target_ent)
        && let Err(err) = nt.command_tx.try_send(NetworkCommand::ClaimObject(obj.0))
    {
        error!(?err, "failed to send claim");
    }
}

pub fn move_grabbed_objects(
    grabbed: Res<GrabbedObjects>,
    time: Res<Time>,
    mut dyn_objs: Query<(&mut LinearVelocity, &mut AngularVelocity), With<DynObjectId>>,
    transforms: Query<&GlobalTransform>,
) {
    let dt = time.delta_secs();
    if dt == 0.0 {
        return;
    }

    for (pointer, grab_state) in &grabbed.0 {
        let Ok((mut obj_vel, mut obj_ang_vel)) = dyn_objs.get_mut(grab_state.entity) else {
            warn!(entity = %grab_state.entity, "object velocity not found");
            continue;
        };

        let Ok(pointer_transform) = transforms.get(*pointer) else {
            warn!(%pointer, "pointer transform not found");
            continue;
        };

        let Ok(obj_transform) = transforms.get(grab_state.entity) else {
            warn!(entity = %grab_state.entity, "object transform not found");
            continue;
        };

        let pointer_tr = pointer_transform.compute_transform();
        let obj_tr = obj_transform.compute_transform();

        // Position tracking
        let target_pos = pointer_tr.translation + pointer_tr.rotation * grab_state.position_offset;
        let delta = target_pos - obj_tr.translation;
        let dist = delta.length();

        obj_vel.0 = if dist < GRAB_DEAD_ZONE {
            Vec3::ZERO
        } else {
            delta * GRAB_SMOOTHING
        };

        // Rotation tracking
        let target_rotation = pointer_tr.rotation * grab_state.rotation_offset;
        let mut rotation_diff = target_rotation * obj_tr.rotation.inverse();

        // Ensure shortest path (quaternion double-cover: q and -q are the same rotation)
        if rotation_diff.w < 0.0 {
            rotation_diff = -rotation_diff;
        }

        let rotation_diff = rotation_diff.normalize();
        let (axis, angle) = rotation_diff.to_axis_angle();

        // Check for valid axis (can be NaN when angle is ~0)
        obj_ang_vel.0 = if angle.abs() < GRAB_ROTATION_DEAD_ZONE || !axis.is_finite() {
            Vec3::ZERO
        } else {
            axis * angle * GRAB_SMOOTHING
        };
    }
}

pub fn setup_grabbed_hooks(world: &mut World) {
    world
        .register_component_hooks::<Grabbed>()
        .on_add(|mut world, context| {
            if let Some(mut gravity) = world.get_mut::<GravityScale>(context.entity) {
                gravity.0 = 0.0;
            } else {
                world
                    .commands()
                    .entity(context.entity)
                    .insert(GravityScale(0.0));
            }
        })
        .on_remove(|mut world, context| {
            if let Some(mut gravity) = world.get_mut::<GravityScale>(context.entity) {
                gravity.0 = 1.0;
            }
        });
}

pub fn update_crosshair_mode(
    mut crosshair: Query<&mut CrosshairMode>,
    ray: Query<&RayHits, With<PrimaryRaycastInput>>,

    grabbable: Query<(), With<DynObjectId>>,
) {
    let Ok(hits) = ray.single() else { return };

    let Ok(mut mode) = crosshair.single_mut() else {
        return;
    };

    if let Some(hit) = hits.iter().next()
        && grabbable.contains(hit.entity)
    {
        *mode = CrosshairMode::Active;
    } else {
        *mode = CrosshairMode::Inactive;
    }
}
