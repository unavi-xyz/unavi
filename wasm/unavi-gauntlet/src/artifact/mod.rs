use std::cell::Cell;

use wired_prelude::prelude::*;

use crate::{
    palette,
    unavi::shapes::api::Cuboid,
    wired::scene::{
        api::self_document,
        types::{
            Prim,
            Xform,
        },
    },
};

const CORE_SIZE: f32 = 0.045;
const ORBITER_SIZE: f32 = 0.018;
const ORBITERS: usize = 3;
const ORBIT_RADIUS: f32 = 0.06;
const SPIN_IDLE: f32 = 0.8;
const SPIN_ACTIVE: f32 = 3.0;
const TILT: f32 = 0.35;

const IDENTITY: Quat = Quat {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 1.0,
};

const fn hidden() -> Xform {
    Xform {
        translation: Vec3::ZERO,
        rotation:    IDENTITY,
        scale:       Vec3::ZERO,
    }
}

/// A small floating artifact: a glowing core with cubes orbiting it. Only shown
/// while a tool is active. Materials are set once; only transforms animate.
pub struct Artifact {
    core:     Prim,
    orbiters: Vec<Prim>,
    spin:     Cell<f32>,
}

impl Artifact {
    #[must_use]
    pub fn new(root: &Prim) -> Self {
        let doc = self_document().expect("self_document");

        let core = Cuboid::new(Vec3::splat(CORE_SIZE)).mesh();
        core.set_material(Some(&palette::solid(palette::ACCENT, 0.6)))
            .ok();
        core.set_xform(Some(hidden())).ok();
        root.add_child(&core).ok();

        let orbiters = (0..ORBITERS)
            .map(|_| {
                let prim = Cuboid::new(Vec3::splat(ORBITER_SIZE)).mesh();
                prim.set_material(Some(&palette::solid(palette::ACCENT, 0.7)))
                    .ok();
                prim.set_xform(Some(hidden())).ok();
                root.add_child(&prim).ok();
                prim
            })
            .collect();

        let _ = doc;
        Self {
            core,
            orbiters,
            spin: Cell::new(0.0),
        }
    }

    pub fn set_color(&self, color: Color) {
        self.core
            .set_material(Some(&palette::solid(color, 0.6)))
            .ok();
        for orbiter in &self.orbiters {
            orbiter.set_material(Some(&palette::solid(color, 0.7))).ok();
        }
    }

    pub fn animate(&self, delta: f32, energy: f32) {
        let speed = energy.mul_add(SPIN_ACTIVE - SPIN_IDLE, SPIN_IDLE);
        let spin = delta.mul_add(speed, self.spin.get());
        self.spin.set(spin);

        self.core
            .set_xform(Some(Xform {
                translation: Vec3::ZERO,
                rotation:    Quat {
                    x: 0.0,
                    y: (spin * 0.5).sin(),
                    z: 0.0,
                    w: (spin * 0.5).cos(),
                },
                scale:       Vec3::ONE,
            }))
            .ok();

        for (i, orbiter) in self.orbiters.iter().enumerate() {
            let phase = spin + i as f32 * std::f32::consts::TAU / ORBITERS as f32;
            orbiter
                .set_xform(Some(Xform {
                    translation: Vec3::new(
                        ORBIT_RADIUS * phase.cos(),
                        ORBIT_RADIUS * TILT * (phase * 2.0).sin(),
                        ORBIT_RADIUS * phase.sin(),
                    ),
                    rotation:    IDENTITY,
                    scale:       Vec3::ONE,
                }))
                .ok();
        }
    }
}
