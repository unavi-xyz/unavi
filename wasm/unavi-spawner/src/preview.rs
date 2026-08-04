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

const PREVIEW_SIZE: f32 = 0.07;
const ABOVE: f32 = 0.13;
const SPIN: f32 = 1.4;

const fn hidden() -> Xform {
    Xform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }
}

/// A small spinning cube floating above the tool artifact, previewing the
/// prefab the spawner will place. A camera-anchored `root` carries orientation
/// so the cube's local spin needs no quaternion composition.
pub struct Preview {
    root:  Prim,
    cube:  Prim,
    spin:  Cell<f32>,
    color: Cell<Option<Color>>,
}

impl Preview {
    #[must_use]
    pub fn new() -> Self {
        let root = self_document()
            .expect("self_document")
            .create_prim()
            .expect("create_prim");
        root.set_xform(Some(hidden())).ok();

        let cuboid = Cuboid::new(Vec3::splat(PREVIEW_SIZE));
        cuboid.set_doc(self_document().expect("self_document"));
        let cube = cuboid.mesh();
        root.add_child(&cube).ok();

        Self {
            root,
            cube,
            spin: Cell::new(0.0),
            color: Cell::new(None),
        }
    }

    pub fn update(&self, cam: &Transform, offset: Vec3, t: f32, color: Color, delta: f32) {
        if self.color.get() != Some(color) {
            self.color.set(Some(color));
            self.cube.set_material(Some(palette::preview(color))).ok();
        }

        self.root
            .set_xform(Some(Xform {
                translation: cam.translation + cam.rotation * offset,
                rotation:    cam.rotation,
                scale:       Vec3::splat(t),
            }))
            .ok();

        let spin = delta.mul_add(SPIN, self.spin.get());
        self.spin.set(spin);
        self.cube
            .set_xform(Some(Xform {
                translation: Vec3::new(0.0, ABOVE, 0.0),
                rotation:    Quat::new(0.0, (spin * 0.5).sin(), 0.0, (spin * 0.5).cos()),
                scale:       Vec3::ONE,
            }))
            .ok();
    }
}
