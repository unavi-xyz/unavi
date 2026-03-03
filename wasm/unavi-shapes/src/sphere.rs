use bevy_math::primitives::Sphere;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{exports::unavi::shapes::api::GuestSphere, wired::scene::types::Mesh};

pub struct SphereWrapped(Sphere);

impl GuestSphere for SphereWrapped {
    fn new(radius: f32) -> Self {
        Self(Sphere { radius })
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(self.0.mesh().build())
    }
}
