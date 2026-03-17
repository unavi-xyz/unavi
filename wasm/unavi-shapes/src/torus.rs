use std::cell::Cell;

use bevy_math::primitives::Torus;
use bevy_mesh::{MeshBuilder, Meshable};

use crate::{exports::unavi::shapes::api::GuestTorus, wired::scene::types::Mesh};

pub struct TorusWrapped {
    inner: Torus,
    minor_resolution: Cell<u32>,
    major_resolution: Cell<u32>,
}

impl GuestTorus for TorusWrapped {
    fn new(minor_radius: f32, major_radius: f32) -> Self {
        Self {
            inner: Torus {
                minor_radius,
                major_radius,
            },
            minor_resolution: Cell::new(24),
            major_resolution: Cell::new(32),
        }
    }

    fn mesh(&self) -> Mesh {
        crate::convert_bevy_mesh(
            self.inner
                .mesh()
                .minor_resolution(self.minor_resolution.get() as usize)
                .major_resolution(self.major_resolution.get() as usize)
                .build(),
        )
    }

    fn minor_resolution(&self) -> u32 {
        self.minor_resolution.get()
    }

    fn set_minor_resolution(&self, value: u32) {
        self.minor_resolution.set(value);
    }

    fn major_resolution(&self) -> u32 {
        self.major_resolution.get()
    }

    fn set_major_resolution(&self, value: u32) {
        self.major_resolution.set(value);
    }
}
