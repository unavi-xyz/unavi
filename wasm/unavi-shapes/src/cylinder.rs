use bevy_math::primitives::Cylinder;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{exports::unavi::shapes::api::GuestCylinder, wired::scene::types::Mesh};

pub struct CylinderWrapped(Cylinder);

impl GuestCylinder for CylinderWrapped {
    fn new(radius: f32, height: f32) -> Self {
        Self(Cylinder {
            radius,
            half_height: height / 2.0,
        })
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(self.0.mesh().build())
    }
}
