use bevy_math::primitives::Cone;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{exports::unavi::shapes::api::GuestCone, wired::scene::types::Mesh};

pub struct ConeWrapped(Cone);

impl GuestCone for ConeWrapped {
    fn new(radius: f32, height: f32) -> Self {
        Self(Cone { radius, height })
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(self.0.mesh().build())
    }
}
