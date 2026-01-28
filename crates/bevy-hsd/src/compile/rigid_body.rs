use avian3d::prelude::RigidBody;
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

        commands.entity(node).insert(kind);
    } else {
        commands.entity(node).remove::<RigidBody>();
    }
}
