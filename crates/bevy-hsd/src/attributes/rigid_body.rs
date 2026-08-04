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

        if let Some(v) = attr.friction {
            if valid_nonneg(v) {
                commands.entity(ent).insert(Friction::new(v as f32));
            } else {
                warn!("rigid_body: friction must be finite and >= 0 (got {v})");
            }
        }

        if let Some(v) = attr.restitution {
            if valid_nonneg(v) {
                commands.entity(ent).insert(Restitution::new(v as f32));
            } else {
                warn!("rigid_body: restitution must be finite and >= 0 (got {v})");
            }
        }

        if let Some(v) = attr.mass {
            if valid_positive(v) {
                commands.entity(ent).insert(Mass(v as f32));
            } else {
                warn!("rigid_body: mass must be finite and > 0 (got {v})");
            }
        }

        if let Some(v) = attr.linear_damping {
            if valid_nonneg(v) {
                commands.entity(ent).insert(LinearDamping(v as f32));
            } else {
                warn!("rigid_body: linear_damping must be finite and >= 0 (got {v})");
            }
        }

        if let Some(v) = attr.angular_damping {
            if valid_nonneg(v) {
                commands.entity(ent).insert(AngularDamping(v as f32));
            } else {
                warn!("rigid_body: angular_damping must be finite and >= 0 (got {v})");
            }
        }
    }
}
