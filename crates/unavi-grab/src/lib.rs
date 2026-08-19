use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    ecs::system::entity_command,
    picking::pointer::PointerInteraction,
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdNamespace,
};
use iroh_docs::NamespaceId;
use unavi_input::{
    crosshair::CrosshairMode,
    pointer::{
        GripPressed,
        GripReleased,
        PointerAnchor,
        PointerKind,
        nearest_hit,
    },
};
use unavi_policy::space::Space;
use unavi_space::{
    anchor::ActiveSpace,
    peer::self_peer_id,
    state::{
        entities,
        replicas,
    },
};

pub struct GrabPlugin;

impl Plugin for GrabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingGrabs>()
            .add_systems(
                Update,
                (
                    on_press,
                    on_release,
                    note_promoted_bodies,
                    start_pending_grabs,
                    reach_grabbed_objects.run_if(unavi_input::capture::scene_has_input),
                    move_grabbed_objects,
                )
                    .chain(),
            )
            .add_systems(FixedUpdate, set_crosshair_mode);
    }
}

/// Backstop for a squeeze whose release never arrives.
const PENDING_GRAB_TIMEOUT: Duration = Duration::from_secs(5);

/// How long a body stays an answer to a squeeze after becoming grabbable.
const PROMOTION_WINDOW: Duration = Duration::from_millis(500);

/// Squeezes that found nothing to carry.
///
/// A script only learns of a grab after the observer has run, so it cannot
/// make something grabbable in time; the squeeze stays pending until a body
/// appears.
#[derive(Resource, Default)]
struct PendingGrabs {
    grabs:    Vec<PendingGrab>,
    /// Bodies that recently became grabbable. A waiting squeeze may latch onto
    /// one under its pointer even when its own ray hit something else. Bounded
    /// to recent promotions so a held squeeze cannot claim a body it merely
    /// swept over.
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

#[derive(Component)]
struct Grabbed {
    pointer:    Entity,
    reach:      f32,
    offset_tra: Vec3,
    offset_rot: Quat,
}

type Docs<'w, 's> = Query<'w, 's, &'static HsdNamespace, With<Hsd>>;

fn begin_grab(
    entity: Entity,
    pointer: Entity,
    reach: f32,
    transforms: &Query<&GlobalTransform>,
    hsd_children: &Query<&HsdChild>,
    docs: &Docs,
    spaces: &Query<&Space>,
    parents: &Query<&ChildOf>,
    active_space: Option<Entity>,
    commands: &mut Commands,
) {
    let Ok(obj_tr) = transforms.get(entity) else {
        warn!(obj = %entity, "object transform not found");
        return;
    };
    let obj_tr = obj_tr.compute_transform();

    let Ok(pointer_tr) = transforms.get(pointer) else {
        warn!(%pointer, "pointer transform not found");
        return;
    };
    let pointer_tr = pointer_tr.compute_transform();

    let offset_tra = pointer_tr.rotation.inverse() * (obj_tr.translation - pointer_tr.translation);
    let offset_rot = pointer_tr.rotation.inverse() * obj_tr.rotation;

    claim_doc_authority(entity, hsd_children, docs, spaces, parents, active_space);

    commands.entity(entity).insert((
        Grabbed {
            pointer,
            reach,
            offset_tra,
            offset_rot,
        },
        GravityScale(0.0),
    ));
}

fn on_press(
    mut presses: MessageReader<GripPressed>,
    transforms: Query<&GlobalTransform>,
    rigid_bodies: Query<&RigidBody>,
    hsd_children: Query<&HsdChild>,
    docs: Docs,
    spaces: Query<&Space>,
    parents: Query<&ChildOf>,
    // Absent in a scene with no space; the authority claim below already
    // skips when there is nothing to claim against.
    active_space: Option<Res<ActiveSpace>>,
    time: Res<Time>,
    mut pending: ResMut<PendingGrabs>,
    mut commands: Commands,
) {
    let active_space = active_space.and_then(|active| active.0);

    for press in presses.read() {
        let target = press.hit.map(|hit| hit.entity);
        let grabbable =
            target.filter(|entity| matches!(rigid_bodies.get(*entity), Ok(RigidBody::Dynamic)));

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

        begin_grab(
            entity,
            press.pointer,
            press.reach,
            &transforms,
            &hsd_children,
            &docs,
            &spaces,
            &parents,
            active_space,
            &mut commands,
        );
    }
}

fn note_promoted_bodies(
    promoted: Query<(Entity, &RigidBody), Changed<RigidBody>>,
    time: Res<Time>,
    mut pending: ResMut<PendingGrabs>,
) {
    let now = time.elapsed();
    pending
        .promoted
        .retain(|(_, at)| now.saturating_sub(*at) <= PROMOTION_WINDOW);

    for (entity, body) in &promoted {
        if matches!(body, RigidBody::Dynamic) {
            pending.promoted.push((entity, now));
        }
    }
}

fn start_pending_grabs(
    transforms: Query<&GlobalTransform>,
    rigid_bodies: Query<&RigidBody>,
    hsd_children: Query<&HsdChild>,
    docs: Docs,
    spaces: Query<&Space>,
    parents: Query<&ChildOf>,
    active_space: Option<Res<ActiveSpace>>,
    time: Res<Time>,
    mut pending: ResMut<PendingGrabs>,
    mut commands: Commands,
) {
    if pending.grabs.is_empty() {
        return;
    }

    let active_space = active_space.and_then(|active| active.0);
    let now = time.elapsed();
    let waiting = std::mem::take(&mut pending.grabs);

    let mut remaining = Vec::with_capacity(waiting.len());
    for grab in waiting {
        if let Some(entity) = target_for(&grab, &rigid_bodies, &transforms, &pending.promoted) {
            begin_grab(
                entity,
                grab.pointer,
                grab.reach,
                &transforms,
                &hsd_children,
                &docs,
                &spaces,
                &parents,
                active_space,
                &mut commands,
            );
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
    rigid_bodies: &Query<&RigidBody>,
    transforms: &Query<&GlobalTransform>,
    promoted: &[(Entity, Duration)],
) -> Option<Entity> {
    if let Some(entity) = grab.entity
        && matches!(rigid_bodies.get(entity), Ok(RigidBody::Dynamic))
    {
        return Some(entity);
    }
    nearest_promoted(grab, transforms, promoted)
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

fn claim_doc_authority(
    entity: Entity,
    hsd_children: &Query<&HsdChild>,
    docs: &Docs,
    spaces: &Query<&Space>,
    parents: &Query<&ChildOf>,
    active_space_entity: Option<Entity>,
) {
    if self_peer_id().is_none() {
        debug!("grab: local peer id not initialized yet, skipping authority claim");
        return;
    }

    let Some((doc_entity, doc_hash)) = resolve_doc(entity, hsd_children, docs) else {
        debug!(
            ?entity,
            "grab: grabbed entity has no HSD doc, skipping authority claim",
        );
        return;
    };

    let space_hash = resolve_space(doc_entity, spaces, parents).or_else(|| {
        let active = active_space_entity?;
        spaces.get(active).ok().map(|s| s.0)
    });

    let Some(space_hash) = space_hash else {
        warn!(
            doc = %doc_hash,
            "grab: no enclosing space and no active space, skipping authority claim",
        );
        return;
    };

    // Only claim authority over a doc already tracked in state. An untracked
    // doc is established by the publish path; claiming here would create
    // presence ahead of that upload.
    if !replicas::has_doc(space_hash, doc_hash) {
        debug!(doc = %doc_hash, "grab: doc not tracked in state, skipping authority claim");
        return;
    }

    info!(doc = %doc_hash, space = %space_hash, "grab: claiming object authority");
    entities::claim_authority(space_hash, doc_hash);
}

fn resolve_doc(
    entity: Entity,
    hsd_children: &Query<&HsdChild>,
    docs: &Docs,
) -> Option<(Entity, NamespaceId)> {
    if let Ok(record) = docs.get(entity) {
        return Some((entity, record.0));
    }
    let child = hsd_children.get(entity).ok()?;
    let record = docs.get(child.0).ok()?;
    Some((child.0, record.0))
}

fn resolve_space(
    doc_entity: Entity,
    spaces: &Query<&Space>,
    parents: &Query<&ChildOf>,
) -> Option<NamespaceId> {
    let mut cursor = Some(doc_entity);
    while let Some(current) = cursor {
        if let Ok(space) = spaces.get(current) {
            return Some(space.0);
        }
        cursor = parents.get(current).ok().map(|p| p.0);
    }
    None
}

fn on_release(
    mut releases: MessageReader<GripReleased>,
    hsd_children: Query<&HsdChild>,
    docs: Docs,
    held: Query<(Entity, &Grabbed)>,
    mut pending: ResMut<PendingGrabs>,
    mut commands: Commands,
) {
    for release in releases.read() {
        pending.grabs.retain(|grab| grab.pointer != release.pointer);

        // Whatever this pointer carries, not whatever it happens to be over —
        // a held object can be dragged clear of its own collider.
        for (entity, _) in held.iter().filter(|(_, g)| g.pointer == release.pointer) {
            if let Some((_, doc_hash)) = resolve_doc(entity, &hsd_children, &docs) {
                entities::release_authority(doc_hash);
            }
            commands
                .entity(entity)
                .queue_silenced(entity_command::remove::<(Grabbed, GravityScale)>());
        }
    }
}

const REACH_STEP: f32 = 0.1;
const MIN_REACH: f32 = 0.3;

/// Scrolls a held object in or out by scaling its hold offset. Capped at the
/// pointer's own reach, so an object can only be scrolled as far as it could
/// have been grabbed.
fn reach_grabbed_objects(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    objects: Query<&mut Grabbed>,
) {
    let notches: f32 = wheel.read().map(|event| event.y.signum()).sum();
    if notches == 0.0 {
        return;
    }

    for mut grabbed in objects {
        let held_at = grabbed.offset_tra.length();
        if held_at <= f32::EPSILON {
            continue;
        }
        let max = grabbed.reach.max(MIN_REACH);
        let wanted = notches.mul_add(REACH_STEP, held_at).clamp(MIN_REACH, max);
        grabbed.offset_tra *= wanted / held_at;
    }
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
            axis * angle * GRAB_SMOOTHING
        };
    }
}

pub fn set_crosshair_mode(
    mut crosshair: Query<&mut CrosshairMode>,
    pointers: Query<(&PointerAnchor, &PointerInteraction)>,
    rigid_bodies: Query<&RigidBody>,
) {
    let Ok(mut mode) = crosshair.single_mut() else {
        return;
    };

    let over_grabbable = pointers
        .iter()
        .find(|(anchor, _)| anchor.0 == PointerKind::Screen)
        .and_then(|(_, interaction)| nearest_hit(interaction))
        .is_some_and(|hit| matches!(rigid_bodies.get(hit.entity), Ok(RigidBody::Dynamic)));

    *mode = if over_grabbable {
        CrosshairMode::Active
    } else {
        CrosshairMode::Inactive
    };
}
