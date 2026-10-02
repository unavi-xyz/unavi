//! What a grabbed body does while a pointer carries it: driven by velocity
//! toward the pointer rather than teleported, so it still collides on the
//! way. A release, a vanished pointer, a demotion off
//! [`RigidBody::Dynamic`](avian3d::prelude::RigidBody::Dynamic) and the
//! held entity despawning all just remove [`Held`]; [`on_held_removed`]
//! is the one place that turns that removal into [`Released`].

use avian3d::prelude::*;
use bevy::{
    ecs::system::SystemParam,
    prelude::*,
};
use unavi_input::{
    action::{
        Action,
        ActionState,
    },
    pointer::GripReleased,
};

use crate::events::{
    Grabbed,
    Released,
};

/// Past this, a grabbed body's velocity is no longer "catching up to the
/// pointer" but "flung by whatever moved the pointer" — a teleport, a portal
/// crossing, a pointer respawn. Clamped rather than smoothed away, so a
/// legitimate fast drag still tracks, just not instantly.
pub const MAX_GRAB_SPEED: f32 = 20.0;

/// See [`MAX_GRAB_SPEED`]; the angular equivalent (radians/second).
pub const MAX_GRAB_ANGULAR_SPEED: f32 = 20.0;

const GRAB_DEAD_ZONE: f32 = 0.001;
const GRAB_ROTATION_DEAD_ZONE: f32 = 0.01;
const GRAB_SMOOTHING: f32 = 10.0;

const REACH_STEP: f32 = 0.1;
const MIN_REACH: f32 = 0.3;

/// What a pointer is carrying, and what to give back once it lets go.
#[derive(Component)]
pub struct Held {
    pub pointer:       Entity,
    pub reach:         f32,
    pub offset_tra:    Vec3,
    pub offset_rot:    Quat,
    /// This entity's own gravity scale before the grab zeroed it. `None`
    /// restores to no [`GravityScale`] at all, rather than assuming `1.0`.
    pub prior_gravity: Option<GravityScale>,
}

/// The queries a grab needs of whatever it might take hold of, shared by
/// every system that starts or matches one.
#[derive(SystemParam)]
pub struct GrabTargets<'w, 's> {
    pub transforms:   Query<'w, 's, &'static GlobalTransform>,
    pub rigid_bodies: Query<'w, 's, &'static RigidBody>,
    gravity:          Query<'w, 's, &'static GravityScale>,
    held:             Query<'w, 's, &'static Held>,
}

/// Takes hold of `entity`, unless it already answers to a pointer.
///
/// Zeroes gravity (remembering what it replaces), inserts [`Held`], and
/// fires [`Grabbed`] so anything that cares what a grab claims beyond
/// physics — a document's hold, say — can react.
pub fn begin_grab(
    entity: Entity,
    pointer: Entity,
    reach: f32,
    targets: &GrabTargets<'_, '_>,
    commands: &mut Commands,
) {
    if targets.held.contains(entity) {
        debug!(%entity, "grab: already held, refusing a second grab");
        return;
    }

    let Ok(obj_tr) = targets.transforms.get(entity) else {
        warn!(obj = %entity, "object transform not found");
        return;
    };
    let obj_tr = obj_tr.compute_transform();

    let Ok(pointer_tr) = targets.transforms.get(pointer) else {
        warn!(%pointer, "pointer transform not found");
        return;
    };
    let pointer_tr = pointer_tr.compute_transform();

    let offset_tra = pointer_tr.rotation.inverse() * (obj_tr.translation - pointer_tr.translation);
    let offset_rot = pointer_tr.rotation.inverse() * obj_tr.rotation;
    let prior_gravity = targets.gravity.get(entity).ok().copied();

    commands.entity(entity).insert((
        Held {
            pointer,
            reach,
            offset_tra,
            offset_rot,
            prior_gravity,
        },
        GravityScale(0.0),
    ));
    commands
        .entity(entity)
        .trigger(move |entity| Grabbed { entity, pointer });
}

/// The one place a grab ends, however it ends: a release, a pointer going
/// away, a demotion off [`RigidBody::Dynamic`], or the held entity
/// despawning outright. Every other end-of-grab path only removes
/// [`Held`]; this restores whatever gravity scale preceded the grab and
/// fires [`Released`].
///
/// Runs before the removal — and, on a despawn, before any of the entity's
/// other components go with it — so `Held`'s own value is still readable
/// here no matter which path ended the grab.
pub fn on_held_removed(trigger: On<Remove, Held>, held: Query<&Held>, mut commands: Commands) {
    let Ok(held) = held.get(trigger.entity) else {
        return;
    };
    let pointer = held.pointer;

    if let Some(gravity) = held.prior_gravity {
        commands.entity(trigger.entity).insert(gravity);
    } else {
        commands.entity(trigger.entity).remove::<GravityScale>();
    }
    commands
        .entity(trigger.entity)
        .trigger(move |entity| Released { entity, pointer });
}

pub fn on_release(
    mut releases: MessageReader<GripReleased>,
    held: Query<(Entity, &Held)>,
    mut commands: Commands,
) {
    for release in releases.read() {
        for (entity, _) in held.iter().filter(|(_, g)| g.pointer == release.pointer) {
            commands.entity(entity).remove::<Held>();
        }
    }
}

/// Scrolls a held object in or out by scaling its hold offset, capped at the
/// pointer's own reach.
pub fn reach_grabbed_objects(state: Res<ActionState>, objects: Query<&mut Held>) {
    let notches = state.delta(Action::Reach).y;
    if notches == 0.0 {
        return;
    }

    for mut held in objects {
        let held_at = held.offset_tra.length();
        if held_at <= f32::EPSILON {
            continue;
        }
        let max = held.reach.max(MIN_REACH);
        let wanted = notches.mul_add(REACH_STEP, held_at).clamp(MIN_REACH, max);
        held.offset_tra *= wanted / held_at;
    }
}

/// Drives every held body's velocity toward its pointer, releasing it
/// instead if the pointer itself is gone.
pub fn move_grabbed_objects(
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    mut objects: Query<(Entity, &Held, &mut LinearVelocity, &mut AngularVelocity)>,
) {
    for (entity, held, mut obj_vel, mut obj_ang_vel) in &mut objects {
        let Ok(pointer_tr) = transforms.get(held.pointer) else {
            commands.entity(entity).remove::<Held>();
            continue;
        };
        let pointer_tr = pointer_tr.compute_transform();

        let Ok(obj_tr) = transforms.get(entity) else {
            continue;
        };
        let obj_tr = obj_tr.compute_transform();

        let target_pos = pointer_tr.translation + pointer_tr.rotation * held.offset_tra;
        let Some(target_pos) = unavi_physics::finite::vec3(target_pos.to_array()) else {
            continue;
        };
        let delta = target_pos - obj_tr.translation;
        let dist = delta.length();

        obj_vel.0 = if dist < GRAB_DEAD_ZONE {
            Vec3::ZERO
        } else {
            (delta * GRAB_SMOOTHING).clamp_length_max(MAX_GRAB_SPEED)
        };

        let target_rotation = pointer_tr.rotation * held.offset_rot;
        let Some(target_rotation) = unavi_physics::finite::quat(target_rotation.to_array()) else {
            continue;
        };
        let mut rotation_diff = target_rotation * obj_tr.rotation.inverse();

        // Ensure shortest path (quaternion double-cover: q and -q are the same
        // rotation)
        if rotation_diff.w < 0.0 {
            rotation_diff = -rotation_diff;
        }

        let rotation_diff = rotation_diff.normalize();
        let (axis, angle) = rotation_diff.to_axis_angle();

        // Check for valid axis (can be NaN when angle is ~0)
        obj_ang_vel.0 = if angle.abs() < GRAB_ROTATION_DEAD_ZONE || !axis.is_finite() {
            Vec3::ZERO
        } else {
            (axis * angle * GRAB_SMOOTHING).clamp_length_max(MAX_GRAB_ANGULAR_SPEED)
        };
    }
}
