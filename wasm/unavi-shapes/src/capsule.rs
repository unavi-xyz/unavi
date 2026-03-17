use std::cell::Cell;

use bevy_math::primitives::Capsule3d;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{
    exports::unavi::shapes::api::GuestCapsule,
    wired::scene::types::{Collider, ColliderCapsule, Mesh},
};

pub struct CapsuleWrapped {
    inner: Capsule3d,
    latitudes: Cell<u32>,
    longitudes: Cell<u32>,
    rings: Cell<u32>,
}

impl GuestCapsule for CapsuleWrapped {
    fn new(radius: f32, height: f32) -> Self {
        Self {
            inner: Capsule3d::new(radius, height),
            latitudes: Cell::new(16),
            longitudes: Cell::new(32),
            rings: Cell::new(0),
        }
    }

    fn collider(&self) -> Collider {
        Collider::Capsule(ColliderCapsule {
            height: self.inner.half_length * 2.0,
            radius: self.inner.radius,
        })
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(
            self.inner
                .mesh()
                .rings(self.rings.get())
                .latitudes(self.latitudes.get())
                .longitudes(self.longitudes.get())
                .build(),
        )
    }

    fn latitudes(&self) -> u32 {
        self.latitudes.get()
    }

    fn set_latitudes(&self, value: u32) {
        self.latitudes.set(value);
    }

    fn longitudes(&self) -> u32 {
        self.longitudes.get()
    }

    fn set_longitudes(&self, value: u32) {
        self.longitudes.set(value);
    }

    fn rings(&self) -> u32 {
        self.rings.get()
    }

    fn set_rings(&self, value: u32) {
        self.rings.set(value);
    }
}
