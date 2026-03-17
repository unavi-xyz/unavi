use std::cell::Cell;

use bevy_math::primitives::Cone;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{exports::unavi::shapes::api::GuestCone, wired::scene::types::Mesh};

pub struct ConeWrapped {
    inner: Cone,
    resolution: Cell<u32>,
}

impl GuestCone for ConeWrapped {
    fn new(radius: f32, height: f32) -> Self {
        Self {
            inner: Cone { radius, height },
            resolution: Cell::new(32),
        }
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(self.inner.mesh().resolution(self.resolution.get()).build())
    }

    fn resolution(&self) -> u32 {
        self.resolution.get()
    }

    fn set_resolution(&self, value: u32) {
        self.resolution.set(value);
    }
}
