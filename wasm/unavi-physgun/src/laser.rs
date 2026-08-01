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

const THIN: f32 = 0.01;

const fn hidden() -> Xform {
    Xform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }
}

/// The rotation mapping +Y onto `dir`, built by hand since the script `Quat`
/// only exposes construction (no axis-angle helpers).
fn align_y_to(dir: Vec3) -> Quat {
    let d = dir.normalize_or_zero();
    let dot = Vec3::Y.dot(d).clamp(-1.0, 1.0);
    if dot > 0.9999 {
        return Quat::IDENTITY;
    }
    if dot < -0.9999 {
        return Quat::new(1.0, 0.0, 0.0, 0.0);
    }
    let axis = Vec3::Y.cross(d).normalize();
    let half = dot.acos() * 0.5;
    let s = half.sin();
    Quat::new(axis.x * s, axis.y * s, axis.z * s, half.cos())
}

/// A thin cuboid stretched between the muzzle and the held object.
pub struct Laser {
    prim:  Prim,
    color: Cell<Option<Color>>,
}

impl Laser {
    #[must_use]
    pub fn new() -> Self {
        let cuboid = Cuboid::new(Vec3::ONE);
        cuboid.set_doc(self_document().expect("self_document"));
        let prim = cuboid.mesh();
        prim.set_xform(Some(hidden())).ok();
        Self {
            prim,
            color: Cell::new(None),
        }
    }

    pub fn show(&self, from: Vec3, to: Vec3, color: Color) {
        if self.color.get() != Some(color) {
            self.color.set(Some(color));
            self.prim.set_material(Some(&palette::beam(color))).ok();
        }
        let delta = to - from;
        let len = delta.length();
        if len < 1.0e-4 {
            self.hide();
            return;
        }
        self.prim
            .set_xform(Some(Xform {
                translation: (from + to) * 0.5,
                rotation:    align_y_to(delta),
                scale:       Vec3::new(THIN, len, THIN),
            }))
            .ok();
    }

    pub fn hide(&self) {
        self.prim.set_xform(Some(hidden())).ok();
    }
}
