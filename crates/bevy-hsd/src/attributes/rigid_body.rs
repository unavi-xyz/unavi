use avian3d::prelude::{
    AngularDamping,
    Friction,
    LinearDamping,
    Mass,
    Restitution,
    RigidBody,
};
use bevy::prelude::*;
use hsd::attributes::{
    Attribute,
    rigid_body::{
        RigidBodyAttr,
        RigidBodyKind,
    },
};

use crate::attributes::{
    AttributeParser,
    ParseError,
    collider::DisabledRigidBody,
    util::{
        valid_nonneg,
        valid_positive,
    },
};

#[derive(Component, Debug, Clone, Copy)]
pub struct RigidBodyData(pub RigidBodyAttr);

pub struct RigidBodyParser;

impl AttributeParser for RigidBodyParser {
    fn key(&self) -> &'static str {
        RigidBodyAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError> {
        match payload {
            Some(payload) => {
                commands
                    .entity(prim)
                    .insert(RigidBodyData(RigidBodyAttr::decode(payload)?));
            }
            None => {
                commands
                    .entity(prim)
                    .remove::<RigidBodyData>()
                    .remove::<RigidBody>()
                    .remove::<DisabledRigidBody>()
                    .remove::<Friction>()
                    .remove::<Restitution>()
                    .remove::<Mass>()
                    .remove::<LinearDamping>()
                    .remove::<AngularDamping>();
            }
        }
        Ok(())
    }
}

pub fn apply_rigid_body(
    changed: Query<(Entity, &RigidBodyData), Changed<RigidBodyData>>,
    mut commands: Commands,
) {
    for (ent, data) in &changed {
        let attr = &data.0;

        // Wait for kind to be committed; never fabricate one (a default Dynamic
        // would let static meshes fall for a frame).
        let Some(kind) = attr.kind else {
            continue;
        };

        let rb = match kind {
            RigidBodyKind::Dynamic => RigidBody::Dynamic,
            RigidBodyKind::Kinematic => RigidBody::Kinematic,
            RigidBodyKind::Static => RigidBody::Static,
        };
        commands.entity(ent).insert(rb);

        apply_scalar(
            &mut commands,
            ent,
            attr.friction,
            valid_nonneg,
            Friction::new,
            "friction",
            ">= 0",
        );
        apply_scalar(
            &mut commands,
            ent,
            attr.restitution,
            valid_nonneg,
            Restitution::new,
            "restitution",
            ">= 0",
        );
        apply_scalar(
            &mut commands,
            ent,
            attr.mass,
            valid_positive,
            Mass,
            "mass",
            "> 0",
        );
        apply_scalar(
            &mut commands,
            ent,
            attr.linear_damping,
            valid_nonneg,
            LinearDamping,
            "linear_damping",
            ">= 0",
        );
        apply_scalar(
            &mut commands,
            ent,
            attr.angular_damping,
            valid_nonneg,
            AngularDamping,
            "angular_damping",
            ">= 0",
        );
    }
}

fn apply_scalar<C: Component>(
    commands: &mut Commands,
    ent: Entity,
    value: Option<f64>,
    valid: fn(f64) -> bool,
    ctor: fn(f32) -> C,
    name: &str,
    constraint: &str,
) {
    let Some(v) = value else { return };
    if valid(v) {
        commands.entity(ent).insert(ctor(v as f32));
    } else {
        warn!("rigid_body: {name} must be finite and {constraint} (got {v})");
    }
}
