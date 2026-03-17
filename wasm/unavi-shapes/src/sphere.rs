use std::cell::Cell;

use bevy_math::primitives::Sphere;
use bevy_mesh::Meshable;

use crate::{
    exports::unavi::shapes::api::GuestSphere,
    wired::scene::types::{Collider, Mesh},
};

pub struct SphereWrapped {
    inner: Sphere,
    subdivisions: Cell<u32>,
}

impl GuestSphere for SphereWrapped {
    fn new(radius: f32) -> Self {
        Self {
            inner: Sphere { radius },
            subdivisions: Cell::new(5),
        }
    }

    fn collider(&self) -> Collider {
        Collider::Sphere(self.inner.radius)
    }

    fn mesh(&self) -> Mesh {
        let bevy_mesh = self
            .inner
            .mesh()
            .ico(self.subdivisions.get())
            .expect("build ico sphere");
        crate::convert_bevy_mesh(bevy_mesh)
    }

    fn subdivisions(&self) -> u32 {
        self.subdivisions.get()
    }

    fn set_subdivisions(&self, value: u32) {
        self.subdivisions.set(value);
    }
}
