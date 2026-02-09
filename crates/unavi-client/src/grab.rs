use std::{collections::HashMap, time::Duration};

use avian3d::prelude::{GravityScale, LinearVelocity};
use bevy::{picking::pointer::PointerId, prelude::*};
use unavi_locomotion::AgentCamera;

use crate::networking::{
    object_publish::{DynObjectId, Grabbed, LocallyOwned},
    thread::{NetworkCommand, NetworkingThread, space::object::outbound::LocalGrabbedObjects},
};

const GRAB_COOLDOWN: Duration = Duration::from_millis(100);
const GRAB_DEAD_ZONE: f32 = 0.001;
const GRAB_SMOOTHING: f32 = 20.0;
const MAX_GRAB_DISTANCE: f32 = 6.0;

/// Tracked 3D transforms of pointers.
/// On desktop, will be the camera's centered frontal ray.
/// In VR, will be hand transforms.
#[derive(Resource, Default)]
pub struct PointerLocations3d(HashMap<PointerId, Transform>);

#[derive(Resource, Default)]
pub struct GrabbedObjects(HashMap<PointerId, GrabState>);

struct GrabState {
    entity: Entity,
    object_id: crate::networking::thread::space::types::object_id::ObjectId,
    offset: Vec3,
}

pub fn handle_grab_click(
    event: On<Pointer<Click>>,
    mut commands: Commands,
    mut grabbed: ResMut<GrabbedObjects>,
    mut local_grabbed: ResMut<LocalGrabbedObjects>,
    locations: Res<PointerLocations3d>,
    time: Res<Time>,
    nt: Res<NetworkingThread>,
    dyn_objs: Query<(&DynObjectId, &Transform), Without<Grabbed>>,
    locally_owned: Query<(), With<LocallyOwned>>,
    mut last_grab: Local<Duration>,
) {
    if event.button != PointerButton::Primary {
        return;
    }

    let now = time.elapsed();
    if now.saturating_sub(*last_grab) < GRAB_COOLDOWN {
        return;
    }

    // Drop the currently grabbed object (if any).
    if let Some(prev) = grabbed.0.remove(&event.pointer_id) {
        commands.entity(prev.entity).remove::<Grabbed>();
        local_grabbed.0.remove(&prev.object_id);

        // Send updated grabbed objects to network thread.
        let _ = nt.command_tx.try_send(NetworkCommand::UpdateGrabbedObjects(
            local_grabbed.0.clone(),
        ));
        return;
    }

    let target_ent = event.event().entity;
    let Ok((obj, obj_tr)) = dyn_objs.get(target_ent) else {
        // Not a dynamic object or already grabbed
        return;
    };

    if event.hit.depth > MAX_GRAB_DISTANCE {
        return;
    }

    let Some(pointer_transform) = locations.0.get(&event.pointer_id) else {
        return;
    };

    info!(pointer = ?event.pointer_id, object = %obj.0.index, "grabbing object");

    // Store offset in pointer's local space so it stays correct when rotating.
    let offset =
        pointer_transform.rotation.inverse() * (obj_tr.translation - pointer_transform.translation);

    grabbed.0.insert(
        event.pointer_id,
        GrabState {
            entity: target_ent,
            object_id: obj.0,
            offset,
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

pub fn track_mouse_pointer(
    mut locations: ResMut<PointerLocations3d>,
    camera: Query<&GlobalTransform, With<AgentCamera>>,
) {
    let Ok(transform) = camera.single() else {
        return;
    };

    locations
        .0
        .insert(PointerId::Mouse, transform.compute_transform());
}

pub fn move_grabbed_objects(
    grabbed: Res<GrabbedObjects>,
    locations: Res<PointerLocations3d>,
    mut dyn_objs: Query<(&Transform, &mut LinearVelocity), With<DynObjectId>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    if dt == 0.0 {
        return;
    }

    for (pointer_id, grab_state) in &grabbed.0 {
        let Some(pointer_tr) = locations.0.get(pointer_id) else {
            warn!(?pointer_id, "pointer transform not found");
            continue;
        };

        let Ok((obj_tr, mut obj_vel)) = dyn_objs.get_mut(grab_state.entity) else {
            warn!(entity = %grab_state.entity, "object transform not found");
            continue;
        };

        let target = pointer_tr.translation + pointer_tr.rotation * grab_state.offset;
        let delta = target - obj_tr.translation;
        let dist = delta.length();

        // Dead zone to prevent jitter when close to target.
        obj_vel.0 = if dist < GRAB_DEAD_ZONE {
            Vec3::ZERO
        } else {
            delta * GRAB_SMOOTHING
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
