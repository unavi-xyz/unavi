use avian3d::prelude::{AngularDamping, LinearDamping, RigidBody};
use bevy::prelude::*;

use crate::stage::Attrs;

pub fn parse_rigid_body_attrs(attrs: &Attrs, node: Entity, commands: &mut Commands) {
    if let Some(kind) = &attrs.rigid_body_kind {
        let kind = match kind.as_str() {
            "dynamic" => RigidBody::Dynamic,
            "static" => RigidBody::Static,
            "kinematic" => RigidBody::Kinematic,
            other => {
                warn!("invalid rigid body kind: {other}");
                RigidBody::default()
            }
        };

        if kind == RigidBody::Dynamic {
            commands
                .entity(node)
                .insert((LinearDamping(0.4), AngularDamping(0.4)));
        }

        commands.entity(node).insert(kind);
    } else {
        commands
            .entity(node)
            .remove::<RigidBody>()
            .remove::<AngularDamping>()
            .remove::<LinearDamping>();
    }
}
