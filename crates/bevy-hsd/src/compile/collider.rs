use avian3d::prelude::Collider;
use bevy::prelude::*;

use crate::stage::Attrs;

pub fn parse_collider_attrs(attrs: &Attrs, node: Entity, commands: &mut Commands) {
    if let Some(shape) = &attrs.collider_shape {
        let mut params = attrs
            .collider_params
            .clone()
            .unwrap_or_default()
            .into_iter();

        let collider = match shape.as_str() {
            "capsule" => {
                let radius = params.next().unwrap_or_default();
                let length = params.next().unwrap_or_default();
                Collider::capsule(radius, length)
            }
            "cone" => {
                let radius = params.next().unwrap_or_default();
                let height = params.next().unwrap_or_default();
                Collider::cone(radius, height)
            }
            "cuboid" => {
                let x_length = params.next().unwrap_or_default();
                let y_length = params.next().unwrap_or_default();
                let z_length = params.next().unwrap_or_default();
                Collider::cuboid(x_length, y_length, z_length)
            }
            "sphere" => {
                let radius = params.next().unwrap_or_default();
                Collider::sphere(radius)
            }
            "trimesh" => {
                // Collider::trimesh(vertices, indices)
                todo!()
            }
            other => {
                warn!("unknown shape: {other}");
                Collider::default()
            }
        };

        commands.entity(node).insert(collider);
    } else {
        commands.entity(node).remove::<Collider>();
    }
}
