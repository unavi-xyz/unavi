//! The body of whatever is in hand, carried in front of the viewer.
//!
//! Law 7 wants a held tool to be visible, and on desktop there is no tracked
//! hand to put one in. This stands in for it: a glowing core with orbiters,
//! wearing the held tool's own colour, shown only while something is held.
//!
//! It is also load-bearing rather than decorative. A tool fires from
//! [`OFFSET`] — the physgun's muzzle *is* this body — so without it a beam
//! starts in empty air and nothing says a tool is running at all.
//!
//! It retires when a tracked hand can hold the tool itself.

use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::{
        api::self_document,
        types::{
            Material,
            Prim,
        },
    },
};

/// Where the held body rides, in the viewer's own frame. The physgun's muzzle
/// is measured from the same place.
pub const OFFSET: Vec3 = Vec3::new(0.22, -0.18, -0.5);

const CORE_SIZE: f32 = 0.045;
const ORBITER_SIZE: f32 = 0.018;
const ORBITERS: usize = 3;
const ORBIT_RADIUS: f32 = 0.06;
const SPIN_IDLE: f32 = 0.8;
const SPIN_ACTIVE: f32 = 3.0;
const TILT: f32 = 0.35;
/// How fast it grows in and out of the hand.
const SPEED: f32 = 5.0;

pub struct Artifact {
    root:     Prim,
    core:     Prim,
    orbiters: Vec<Prim>,
    spin:     f32,
    /// How far out it is, 0 to 1, so appearing and going away are the same
    /// motion run in opposite directions.
    out:      f32,
}

impl Artifact {
    pub fn new() -> anyhow::Result<Self> {
        let doc = self_document()?;
        let root = doc.create_prim()?;
        root.set_xform(Some(hidden()))?;

        let core = Cuboid::new(Vec3::splat(CORE_SIZE)).mesh();
        core.set_xform(Some(hidden()))?;
        root.add_child(&core)?;

        let mut orbiters = Vec::with_capacity(ORBITERS);
        for _ in 0..ORBITERS {
            let orbiter = Cuboid::new(Vec3::splat(ORBITER_SIZE)).mesh();
            orbiter.set_xform(Some(hidden()))?;
            root.add_child(&orbiter)?;
            orbiters.push(orbiter);
        }

        Ok(Self {
            root,
            core,
            orbiters,
            spin: 0.0,
            out: 0.0,
        })
    }

    /// Dresses it in the held tool's colour. Materials are written only when
    /// the tool changes, never per frame.
    pub fn wear(&self, color: Color) {
        self.core.set_material(Some(lit(color, 0.6))).ok();
        for orbiter in &self.orbiters {
            orbiter.set_material(Some(lit(color, 0.75))).ok();
        }
    }

    /// Rides the viewer, spinning faster the further out it is.
    pub fn update(&mut self, eye: &Transform, held: bool, delta: f32) -> anyhow::Result<()> {
        let step = delta * SPEED;
        self.out = if held {
            (self.out + step).min(1.0)
        } else {
            (self.out - step).max(0.0)
        };

        self.root.set_xform(Some(Transform {
            translation: eye.translation + eye.rotation * OFFSET,
            rotation:    eye.rotation,
            scale:       Vec3::splat(self.out),
        }))?;
        if self.out <= 0.0 {
            return Ok(());
        }

        self.spin = delta.mul_add(
            self.out.mul_add(SPIN_ACTIVE - SPIN_IDLE, SPIN_IDLE),
            self.spin,
        );
        self.core.set_xform(Some(Transform {
            translation: Vec3::ZERO,
            rotation:    spun(self.spin),
            scale:       Vec3::ONE,
        }))?;

        for (index, orbiter) in self.orbiters.iter().enumerate() {
            let phase = self.spin + index as f32 * std::f32::consts::TAU / ORBITERS as f32;
            orbiter.set_xform(Some(Transform {
                translation: Vec3::new(
                    ORBIT_RADIUS * phase.cos(),
                    ORBIT_RADIUS * TILT * (phase * 2.0).sin(),
                    ORBIT_RADIUS * phase.sin(),
                ),
                rotation:    Quat::IDENTITY,
                scale:       Vec3::ONE,
            }))?;
        }
        Ok(())
    }
}

const fn hidden() -> Transform {
    Transform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }
}

fn spun(angle: f32) -> Quat {
    Quat::new(0.0, (angle * 0.5).sin(), 0.0, (angle * 0.5).cos())
}

const fn lit(color: Color, glow: f32) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   None,
        base_color:   Some(color),
        double_sided: None,
        emissive:     Some(Color {
            r: color.r * glow,
            g: color.g * glow,
            b: color.b * glow,
            a: 1.0,
        }),
        metallic:     Some(0.0),
        roughness:    Some(0.5),
    }
}
