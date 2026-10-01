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
    body::DisabledRigidBody,
    finite::{
        nonneg,
        positive,
    },
};

pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    match payload {
        Some(payload) => build(commands, prim, &RigidBodyAttr::decode(payload)?),
        None => {
            commands.entity(prim).remove::<(
                RigidBody,
                Position,
                Rotation,
                DisabledRigidBody,
                Friction,
                Restitution,
                Mass,
                LinearDamping,
                AngularDamping,
            )>();
        }
    }
    Ok(())
}

/// Builds nothing until `kind` is set, so a static body never falls for a
/// frame as a default `Dynamic`.
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
        nonneg,
        Friction::new,
        "friction",
        ">= 0",
    );
    apply_scalar(
        commands,
        ent,
        attr.restitution,
        nonneg,
        Restitution::new,
        "restitution",
        ">= 0",
    );
    apply_scalar(commands, ent, attr.mass, positive, Mass, "mass", "> 0");
    apply_scalar(
        commands,
        ent,
        attr.linear_damping,
        nonneg,
        LinearDamping,
        "linear_damping",
        ">= 0",
    );
    apply_scalar(
        commands,
        ent,
        attr.angular_damping,
        nonneg,
        AngularDamping,
        "angular_damping",
        ">= 0",
    );
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
