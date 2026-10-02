use avian3d::prelude::{
    AngularDamping,
    Friction,
    LinearDamping,
    Mass,
    Position,
    Restitution,
    RigidBody,
    Rotation,
};
use bevy::prelude::*;
use hsd::{
    attributes::rigid_body::{
        RigidBodyAttr,
        RigidBodyKind,
    },
    property::Payload,
};
use unavi_physics::{
    PhysicsLimits,
    degenerate::Parked,
    finite::{
        nonnegative_length,
        positive_length,
    },
};

use crate::prim::Prim;

/// Every component this attribute can leave on a prim. A full detach
/// (`apply`'s `None` branch) and a refused insertion
/// ([`enforce_rigid_body_limit`]) both remove exactly this set, so neither
/// can leave the other's leftovers (a stale `Mass`, a parked pose) behind.
type RigidBodyComponents = (
    RigidBody,
    Position,
    Rotation,
    Parked,
    Friction,
    Restitution,
    Mass,
    LinearDamping,
    AngularDamping,
);

pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    match payload {
        Some(payload) => build(commands, prim, &RigidBodyAttr::decode(payload)?),
        None => {
            commands.entity(prim).remove::<RigidBodyComponents>();
        }
    }
    Ok(())
}

/// Builds nothing until `kind` is set, so a static body never falls for a
/// frame as a default `Dynamic`. The body limit is enforced separately, in
/// [`enforce_rigid_body_limit`], once this insertion has landed.
fn build(commands: &mut Commands, ent: Entity, attr: &RigidBodyAttr) {
    let Some(kind) = attr.kind else { return };

    let rb = match kind {
        RigidBodyKind::Dynamic => RigidBody::Dynamic,
        RigidBodyKind::Kinematic => RigidBody::Kinematic,
        RigidBodyKind::Static => RigidBody::Static,
    };
    commands.entity(ent).insert(rb);

    apply_scalar(
        commands,
        ent,
        attr.friction,
        nonnegative_length,
        Friction::new,
        "friction",
        ">= 0",
    );
    apply_scalar(
        commands,
        ent,
        attr.restitution,
        nonnegative_length,
        Restitution::new,
        "restitution",
        ">= 0",
    );
    apply_scalar(
        commands,
        ent,
        attr.mass,
        positive_length,
        Mass,
        "mass",
        "> 0",
    );
    apply_scalar(
        commands,
        ent,
        attr.linear_damping,
        nonnegative_length,
        LinearDamping,
        "linear_damping",
        ">= 0",
    );
    apply_scalar(
        commands,
        ent,
        attr.angular_damping,
        nonnegative_length,
        AngularDamping,
        "angular_damping",
        ">= 0",
    );
}

/// Caps how many prim-driven rigid bodies the solver carries, counting only
/// prims (`With<Prim>`): a local agent's rig, a portal body, or anything
/// else Rust code spawns directly must not shrink a scene's own budget or be
/// refused on the scene's behalf. `rigid_body`'s own `apply` only has a bare
/// `Commands`, with no query to count against, so this runs right after it
/// in the same chain, touching only prims added this frame
/// (`Added<RigidBody>`); it never revisits a body that landed on an earlier
/// frame. Which of a same-frame batch gets refused once the cap is reached
/// follows query iteration order (archetype order), not insertion order —
/// exceeding the cap at all is the anomaly being guarded against, not
/// fairness among simultaneous arrivals.
pub fn enforce_rigid_body_limit(
    limits: Res<PhysicsLimits>,
    added: Query<Entity, (Added<RigidBody>, With<Prim>)>,
    all: Query<(), (With<RigidBody>, With<Prim>)>,
    mut commands: Commands,
) {
    if added.is_empty() {
        return;
    }
    let existing = all.iter().count().saturating_sub(added.iter().count());
    let mut count = existing;
    for entity in &added {
        count += 1;
        if count > limits.max_bodies {
            warn!(
                %entity,
                max = limits.max_bodies,
                "rigid_body: scene exceeded the body limit; refusing"
            );
            commands.entity(entity).remove::<RigidBodyComponents>();
        }
    }
}

/// Inserts `C` when `value` is present and valid, removes it otherwise.
fn apply_scalar<C: Component>(
    commands: &mut Commands,
    ent: Entity,
    value: Option<f64>,
    valid: fn(f32) -> bool,
    ctor: fn(f32) -> C,
    name: &str,
    constraint: &str,
) {
    match value {
        Some(v) if valid(v as f32) => {
            commands.entity(ent).insert(ctor(v as f32));
        }
        Some(v) => {
            warn!("rigid_body: {name} must be finite and {constraint} (got {v})");
            commands.entity(ent).remove::<C>();
        }
        None => {
            commands.entity(ent).remove::<C>();
        }
    }
}
