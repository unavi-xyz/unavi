use std::cell::Cell;

use bevy_math::primitives::Cylinder;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{
    exports::unavi::shapes::api::GuestCylinder,
    wired::scene::types::{Collider, ColliderCylinder, Mesh},
};

pub struct CylinderWrapped {
    inner: Cylinder,
    resolution: Cell<u32>,
    segments: Cell<u32>,
}

impl GuestCylinder for CylinderWrapped {
    fn new(radius: f32, height: f32) -> Self {
        Self {
            inner: Cylinder {
                radius,
                half_height: height / 2.0,
            },
            resolution: Cell::new(32),
            segments: Cell::new(1),
        }
    }

    fn collider(&self) -> Collider {
        Collider::Cylinder(ColliderCylinder {
            height: self.inner.half_height * 2.0,
            radius: self.inner.radius,
        })
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(
            self.inner
                .mesh()
                .resolution(self.resolution.get())
                .segments(self.segments.get())
                .build(),
        )
    }

    fn resolution(&self) -> u32 {
        self.resolution.get()
    }

    fn set_resolution(&self, value: u32) {
        self.resolution.set(value);
    }

    fn segments(&self) -> u32 {
        self.segments.get()
    }

    fn set_segments(&self, value: u32) {
        self.segments.set(value);
    }
}
