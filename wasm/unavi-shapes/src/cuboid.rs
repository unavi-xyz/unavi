use bevy_math::primitives::Cuboid;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{
    exports::unavi::shapes::api::GuestCuboid,
    wired::scene::types::{Collider, Mesh, Vec3 as WitVec3},
};

#[derive(Default)]
pub struct CuboidWrapped(Cuboid);

impl GuestCuboid for CuboidWrapped {
    fn new(x_length: f32, y_length: f32, z_length: f32) -> Self {
        Self(Cuboid::new(x_length, y_length, z_length))
    }

    fn collider(&self) -> Collider {
        Collider::Cuboid(WitVec3 {
            x: self.0.half_size.x * 2.0,
            y: self.0.half_size.y * 2.0,
            z: self.0.half_size.z * 2.0,
        })
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(self.0.mesh().build())
    }
}
