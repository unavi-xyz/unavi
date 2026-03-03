use bevy_math::primitives::Capsule3d;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{exports::unavi::shapes::api::GuestCapsule, wired::scene::types::Mesh};

pub struct CapsuleWrapped(Capsule3d);

impl GuestCapsule for CapsuleWrapped {
    fn new(radius: f32, height: f32) -> Self {
        Self(Capsule3d::new(radius, height))
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(self.0.mesh().build())
    }
}
