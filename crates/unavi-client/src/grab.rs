use avian3d::prelude::*;
use bevy::prelude::*;
use unavi_input::{SqueezeDown, SqueezeUp, crosshair::CrosshairMode, raycast::PrimaryRaycastInput};

pub struct GrabPlugin;

impl Plugin for GrabPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_squeeze_down)
            .add_observer(on_squeeze_up)
            .add_systems(Update, move_grabbed_objects)
            .add_systems(FixedUpdate, set_crosshair_mode);
    }
}

#[derive(Component)]
struct Grabbed {
    pointer: Entity,
    offset_tra: Vec3,
    offset_rot: Quat,
}

fn on_squeeze_down(
    trigger: On<SqueezeDown>,
    transforms: Query<&GlobalTransform>,
    rigid_bodies: Query<&RigidBody>,
    mut commands: Commands,
) {
    let Some(entity) = trigger.entity else {
        return;
    };

    if !matches!(rigid_bodies.get(entity), Ok(RigidBody::Dynamic)) {
        return;
    }

    let Ok(obj_tr) = transforms.get(entity) else {
        warn!(obj = %entity, "object transform not found");
        return;
    };
    let obj_tr = obj_tr.compute_transform();

    let Ok(pointer_tr) = transforms.get(trigger.pointer) else {
        warn!(pointer = %trigger.pointer, "pointer transform not found");
        return;
    };
    let pointer_tr = pointer_tr.compute_transform();

    let offset_tra = pointer_tr.rotation.inverse() * (obj_tr.translation - pointer_tr.translation);
    let offset_rot = pointer_tr.rotation.inverse() * obj_tr.rotation;

    // TODO claim / broadcast over network within unavi-space

    commands.entity(entity).insert((
        Grabbed {
            pointer: trigger.pointer,
            offset_tra,
            offset_rot,
        },
        GravityScale(0.0),
    ));
}

fn on_squeeze_up(trigger: On<SqueezeUp>, mut commands: Commands) {
    let Some(entity) = trigger.entity else {
        return;
    };
    commands.entity(entity).remove::<(Grabbed, GravityScale)>();
}

const GRAB_DEAD_ZONE: f32 = 0.001;
const GRAB_ROTATION_DEAD_ZONE: f32 = 0.01;
const GRAB_SMOOTHING: f32 = 10.0;

fn move_grabbed_objects(
    transforms: Query<&GlobalTransform>,
    objects: Query<(Entity, &Grabbed, &mut LinearVelocity, &mut AngularVelocity)>,
) {
    for (entity, grabbed, mut obj_vel, mut obj_ang_vel) in objects {
        let Ok(pointer_tr) = transforms.get(grabbed.pointer) else {
            warn!(pointer = %grabbed.pointer, "pointer transform not found");
            continue;
        };
        let pointer_tr = pointer_tr.compute_transform();

        let Ok(obj_tr) = transforms.get(entity) else {
            continue;
        };
        let obj_tr = obj_tr.compute_transform();

        let target_pos = pointer_tr.translation + pointer_tr.rotation * grabbed.offset_tra;
        let delta = target_pos - obj_tr.translation;
        let dist = delta.length();

        obj_vel.0 = if dist < GRAB_DEAD_ZONE {
            Vec3::ZERO
        } else {
            delta * GRAB_SMOOTHING
        };

        let target_rotation = pointer_tr.rotation * grabbed.offset_rot;
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

pub fn set_crosshair_mode(
    mut crosshair: Query<&mut CrosshairMode>,
    ray: Query<&RayHits, With<PrimaryRaycastInput>>,
    rigid_bodies: Query<&RigidBody>,
) {
    let Ok(hits) = ray.single() else { return };

    let Ok(mut mode) = crosshair.single_mut() else {
        return;
    };

    if let Some(hit) = hits.iter().next()
        && matches!(rigid_bodies.get(hit.entity), Ok(RigidBody::Dynamic))
    {
        *mode = CrosshairMode::Active;
    } else {
        *mode = CrosshairMode::Inactive;
    }
}
