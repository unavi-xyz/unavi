use bevy_math::primitives::Torus;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{exports::unavi::shapes::api::GuestTorus, wired::scene::types::Mesh};

pub struct TorusWrapped(Torus);

impl GuestTorus for TorusWrapped {
    fn new(minor_radius: f32, major_radius: f32) -> Self {
        Self(Torus {
            minor_radius,
            major_radius,
        })
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(self.0.mesh().build())
    }
}
