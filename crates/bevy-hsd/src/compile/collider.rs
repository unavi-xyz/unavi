use avian3d::prelude::Collider;
use bevy::prelude::*;

use crate::data::HsdCollider;

pub fn parse_collider_data(
    mut commands: Commands,
    colliders: Query<(Entity, &HsdCollider), Added<HsdCollider>>,
) {
    for (ent, data) in &colliders {
        let mut size = data.size.iter().copied();

        #[expect(clippy::cast_possible_truncation)]
        let collider = match data.shape.as_str() {
            "capsule" => {
                let radius = size.next().unwrap_or_default() as f32;
                let length = size.next().unwrap_or_default() as f32;
                Collider::capsule(radius, length)
            }
            "cone" => {
                let radius = size.next().unwrap_or_default() as f32;
                let height = size.next().unwrap_or_default() as f32;
                Collider::cone(radius, height)
            }
            "cuboid" => {
                let x = size.next().unwrap_or_default() as f32;
                let y = size.next().unwrap_or_default() as f32;
                let z = size.next().unwrap_or_default() as f32;
                Collider::cuboid(x, y, z)
            }
            "sphere" => {
                let radius = size.next().unwrap_or_default() as f32;
                Collider::sphere(radius)
            }
            other => {
                warn!("unknown collider shape: {other}");
                Collider::default()
            }
        };

        debug!("compiled collider {ent}");
        commands.entity(ent).insert(collider);
    }
}
