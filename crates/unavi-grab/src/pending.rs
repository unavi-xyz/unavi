//! A squeeze onto something not yet grabbable waits, so a script can answer
//! a grab by making the thing grabbable after the fact.

use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use unavi_input::{
    crosshair::Grabbable,
    pointer::{
        GripPressed,
        GripReleased,
    },
};

use crate::held::{
    GrabTargets,
    Held,
    begin_grab,
};

/// Backstop for a squeeze whose release never arrives.
const PENDING_GRAB_TIMEOUT: Duration = Duration::from_secs(5);

/// How long a body stays an answer to a squeeze after becoming grabbable.
const PROMOTION_WINDOW: Duration = Duration::from_millis(500);

/// Squeezes that found nothing to carry.
///
/// A script only learns of a grab after the observer has run, so the squeeze
/// stays pending until a grabbable body appears under its pointer.
#[derive(Resource, Default)]
pub struct PendingGrabs {
    grabs:    Vec<PendingGrab>,
    /// Bodies that recently became grabbable. Bounded to recent promotions so
    /// a held squeeze cannot claim a body it merely swept over.
    promoted: Vec<(Entity, Duration)>,
}

struct PendingGrab {
    /// None when the squeeze landed on nothing at all.
    entity:  Option<Entity>,
    pointer: Entity,
    ray:     Ray3d,
    reach:   f32,
    since:   Duration,
}

pub fn on_press(
    mut presses: MessageReader<GripPressed>,
    targets: GrabTargets,
    time: Res<Time>,
    mut pending: ResMut<PendingGrabs>,
    mut commands: Commands,
) {
    for press in presses.read() {
        let target = press.hit.map(|hit| hit.entity);
        let grabbable = target
            .filter(|entity| matches!(targets.rigid_bodies.get(*entity), Ok(RigidBody::Dynamic)));

        let Some(entity) = grabbable else {
            pending.grabs.push(PendingGrab {
                entity:  target,
                pointer: press.pointer,
                ray:     press.ray,
                reach:   press.reach,
                since:   time.elapsed(),
            });
            continue;
        };

        begin_grab(entity, press.pointer, press.reach, &targets, &mut commands);
    }
}

/// Drops a squeeze still waiting on this pointer once it lets go.
pub fn cancel_pending_on_release(
    mut releases: MessageReader<GripReleased>,
    mut pending: ResMut<PendingGrabs>,
) {
    for release in releases.read() {
        pending.grabs.retain(|grab| grab.pointer != release.pointer);
    }
}

/// Also keeps [`Grabbable`] in step with [`RigidBody::Dynamic`], so the
/// crosshair can show its ring without knowing what a rigid body is, and
/// ends a grab a body demoted away from `Dynamic` should not keep (removing
/// [`Held`] releases it through [`crate::held::on_held_removed`]).
pub fn note_promoted_bodies(
    promoted: Query<(Entity, &RigidBody), Changed<RigidBody>>,
    time: Res<Time>,
    mut pending: ResMut<PendingGrabs>,
    mut commands: Commands,
) {
    let now = time.elapsed();
    pending
        .promoted
        .retain(|(_, at)| now.saturating_sub(*at) <= PROMOTION_WINDOW);

    for (entity, body) in &promoted {
        if matches!(body, RigidBody::Dynamic) {
            pending.promoted.push((entity, now));
            commands.entity(entity).insert(Grabbable);
        } else {
            commands.entity(entity).remove::<Grabbable>();
            commands.entity(entity).remove::<Held>();
        }
    }
}

/// A body stops being grabbable the moment it stops being a rigid body at
/// all, not only when [`RigidBody`] changes value: [`Changed`] never fires on
/// a removal, so a scan for bodies left without one is the only path that
/// notices. A `Remove` observer cannot tell a live removal from a despawn,
/// and would queue the removals against the despawned entity.
pub fn drop_grabbable_without_body(
    stale: Query<
        (Entity, Has<Grabbable>, Has<Held>),
        (Without<RigidBody>, Or<(With<Grabbable>, With<Held>)>),
    >,
    mut commands: Commands,
) {
    for (entity, grabbable, held) in &stale {
        if grabbable {
            commands.entity(entity).remove::<Grabbable>();
        }
        if held {
            commands.entity(entity).remove::<Held>();
        }
    }
}

pub fn start_pending_grabs(
    targets: GrabTargets,
    time: Res<Time>,
    mut pending: ResMut<PendingGrabs>,
    mut commands: Commands,
) {
    if pending.grabs.is_empty() {
        return;
    }

    let now = time.elapsed();
    let waiting = std::mem::take(&mut pending.grabs);

    let mut remaining = Vec::with_capacity(waiting.len());
    for grab in waiting {
        if let Some(entity) = target_for(&grab, &targets, &pending.promoted) {
            begin_grab(entity, grab.pointer, grab.reach, &targets, &mut commands);
        } else if now.saturating_sub(grab.since) <= PENDING_GRAB_TIMEOUT {
            remaining.push(grab);
        }
    }
    pending.grabs = remaining;
}

/// Maximum off-axis tangent at which a promoted body still answers a waiting
/// squeeze. Generous because the pointer moves on while the promotion crosses
/// into the ECS.
const GRAB_CATCH_TANGENT: f32 = 0.2;

fn target_for(
    grab: &PendingGrab,
    targets: &GrabTargets<'_, '_>,
    promoted: &[(Entity, Duration)],
) -> Option<Entity> {
    if let Some(entity) = grab.entity
        && matches!(targets.rigid_bodies.get(entity), Ok(RigidBody::Dynamic))
    {
        return Some(entity);
    }
    nearest_promoted(grab, &targets.transforms, promoted)
}

fn nearest_promoted(
    grab: &PendingGrab,
    transforms: &Query<&GlobalTransform>,
    promoted: &[(Entity, Duration)],
) -> Option<Entity> {
    let origin = grab.ray.origin;
    let direction = *grab.ray.direction;

    promoted
        .iter()
        .filter_map(|(entity, _)| {
            let offset = transforms.get(*entity).ok()?.translation() - origin;
            let along = offset.dot(direction);
            if along <= 0.0 || along > grab.reach {
                return None;
            }
            let off_axis = (offset - direction * along).length();
            (off_axis <= along * GRAB_CATCH_TANGENT).then_some((*entity, along))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(entity, _)| entity)
}
