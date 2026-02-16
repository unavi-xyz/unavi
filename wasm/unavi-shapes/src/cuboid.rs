use bevy_math::primitives::Cuboid;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{exports::unavi::shapes::api::GuestCuboid, wired::scene::types::Mesh};

#[derive(Default)]
pub struct CuboidWrapped(Cuboid);

impl GuestCuboid for CuboidWrapped {
    fn new(x_length: f32, y_length: f32, z_length: f32) -> Self {
        Self(Cuboid::new(x_length, y_length, z_length))
    }

    fn mesh(&self) -> Mesh {
        let mesh = self.0.mesh().build();
        crate::convert_bevy_mesh(mesh)
    }
}
