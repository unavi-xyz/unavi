use avian3d::prelude::{AngularDamping, LinearDamping, RigidBody};
use bevy::prelude::*;

use crate::data::HsdRigidBody;

pub fn parse_rigid_body_data(
    mut commands: Commands,
    bodies: Query<(Entity, &HsdRigidBody), Added<HsdRigidBody>>,
) {
    for (ent, data) in &bodies {
        let kind = match data.kind.as_str() {
            "dynamic" => RigidBody::Dynamic,
            "fixed" => RigidBody::Static,
            "kinematic" => RigidBody::Kinematic,
            other => {
                warn!("invalid rigid body kind: {other}");
                RigidBody::default()
            }
        };

        let mut ent = commands.entity(ent);
        ent.insert(kind);

        if kind == RigidBody::Dynamic {
            #[expect(clippy::cast_possible_truncation)]
            let linear = data.linear_damping.map_or(0.2, |v| v as f32);
            #[expect(clippy::cast_possible_truncation)]
            let angular = data.angular_damping.map_or(0.2, |v| v as f32);
            ent.insert((LinearDamping(linear), AngularDamping(angular)));
        }
    }
}
