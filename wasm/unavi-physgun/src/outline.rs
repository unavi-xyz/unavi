use std::cell::RefCell;

use wired_prelude::prelude::*;

use crate::{
    palette,
    unavi::shapes::api::{
        Capsule,
        Cuboid,
        Cylinder,
        Sphere,
    },
    wired::scene::{
        api::self_document,
        types::{
            Collider,
            GraphValue,
            Prim,
        },
    },
};

/// Shell offset from the prop's surface, fixed so the rim does not grow with
/// the prop.
const MARGIN: f32 = 0.01;
/// A shell grows by [`MARGIN`] on each side, so an extent grows by twice it.
const EXTENT_MARGIN: f32 = MARGIN * 2.0;

const TEMPLATE_PRIM_NAME: &str = "glow_template";
/// Tint input index.
const TINT_INPUT: u16 = 0;

const fn hidden() -> Transform {
    Transform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }
}

/// Builds a mesh matching `collider`, grown by [`MARGIN`]. `ConvexHull` and
/// `Trimesh` keep their geometry where a script cannot read it, so those
/// props get no highlight.
fn shell_mesh(collider: &Collider) -> Option<Prim> {
    let doc = self_document().ok()?;
    match collider {
        Collider::Cuboid(size) => {
            let grown = Vec3::new(
                size.x + EXTENT_MARGIN,
                size.y + EXTENT_MARGIN,
                size.z + EXTENT_MARGIN,
            );
            let shape = Cuboid::new(grown);
            shape.set_doc(doc);
            Some(shape.mesh())
        }
        Collider::Sphere(radius) => {
            let shape = Sphere::new(radius + MARGIN);
            shape.set_doc(doc);
            Some(shape.mesh())
        }
        Collider::Capsule(c) => {
            let shape = Capsule::new(c.radius + MARGIN, c.height + EXTENT_MARGIN);
            shape.set_doc(doc);
            Some(shape.mesh())
        }
        Collider::Cylinder(c) => {
            let shape = Cylinder::new(c.radius + MARGIN, c.height + EXTENT_MARGIN);
            shape.set_doc(doc);
            Some(shape.mesh())
        }
        Collider::ConvexHull | Collider::Trimesh => None,
    }
}

/// An additive rim shell tracking the held prop, owned by this script so the
/// prop's own material is never touched; minted per grab because its mesh
/// depends on the prop's shape.
#[derive(Default)]
pub struct Outline(RefCell<Option<Prim>>);

impl Outline {
    /// Mints a shell for `collider`. A prop whose shape cannot be read gets
    /// no shell, and [`Self::track`] then does nothing.
    pub fn attach(&self, collider: &Collider, color: Color) {
        self.clear();

        let Some(prim) = shell_mesh(collider) else {
            return;
        };
        prim.set_xform(Some(hidden())).ok();

        match self_document()
            .ok()
            .map(|doc| doc.prims())
            .and_then(|prims| {
                prims
                    .into_iter()
                    .find(|p| p.name().is_some_and(|n| n == TEMPLATE_PRIM_NAME))
            }) {
            Some(template) => {
                prim.set_relationship("material:binding", Some(&template.id()))
                    .ok();
                prim.set_graph_overrides(&[(
                    TINT_INPUT,
                    GraphValue::Color(palette::beam_tint(color)),
                )])
                .ok();
            }
            None => eprintln!("physgun: HSD missing {TEMPLATE_PRIM_NAME} prim; prop unhighlighted"),
        }

        *self.0.borrow_mut() = Some(prim);
    }

    /// Matches the prop's pose at render rate; a shell lagging a frame behind
    /// reads as sliding off the object.
    pub fn track(&self, body: &Transform) {
        if let Some(prim) = self.0.borrow().as_ref() {
            prim.set_xform(Some(Transform {
                translation: body.translation,
                rotation:    body.rotation,
                scale:       Vec3::ONE,
            }))
            .ok();
        }
    }

    /// Removes the shell prim outright rather than hiding it: its mesh only
    /// fits the prop it was minted for, and the next grab mints its own.
    pub fn clear(&self) {
        let Some(prim) = self.0.borrow_mut().take() else {
            return;
        };
        if let Ok(doc) = self_document() {
            doc.remove_prim(&prim).ok();
        }
    }
}
