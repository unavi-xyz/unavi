//! An additive rim shell tracking the grabbed prop, minted per grab because
//! its mesh depends on the prop's shape.

use std::cell::RefCell;

use wired_guest::{
    math::{
        Color,
        Transform,
        Vec3,
    },
    xform::hidden,
};

use crate::{
    find_one,
    unavi::shapes::api::{
        Capsule,
        Cuboid,
        Cylinder,
        Sphere,
    },
    wired::{
        scene::{
            document::{
                Document,
                Layer,
                script_document,
            },
            properties::{
                Collider,
                Property,
                Relation,
            },
        },
        shading::graph::{
            GraphValue,
            set_overrides,
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

/// Builds a mesh matching `collider`, grown by [`MARGIN`]. `ConvexHull` and
/// `Trimesh` keep their geometry where a script cannot read it, so those
/// props get no highlight.
fn shell_mesh(collider: &Collider) -> Option<(u64, u64)> {
    let mesh = match collider {
        Collider::Cuboid(size) => Cuboid::new(*size + Vec3::splat(EXTENT_MARGIN)).mesh(),
        Collider::Sphere(radius) => Sphere::new(radius + MARGIN).mesh(),
        Collider::Capsule(c) => Capsule::new(c.radius + MARGIN, c.height + EXTENT_MARGIN).mesh(),
        Collider::Cylinder(c) => Cylinder::new(c.radius + MARGIN, c.height + EXTENT_MARGIN).mesh(),
        Collider::ConvexHull | Collider::Trimesh => return None,
    };
    match mesh {
        Ok(prim) => Some(prim),
        Err(err) => {
            eprintln!("physgun: outline mesh: {err:?}");
            None
        }
    }
}

/// An additive rim shell tracking the held prop, owned by this script so the
/// prop's own material is never touched.
pub struct Outline {
    doc:  Document,
    prim: RefCell<Option<(u64, u64)>>,
}

impl Outline {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            doc:  script_document()?,
            prim: RefCell::new(None),
        })
    }

    /// Mints a shell for `collider`. A prop whose shape cannot be read gets
    /// no shell, and [`Self::track`] then does nothing.
    pub fn attach(&self, collider: &Collider, color: Color) {
        self.clear();

        let Some(prim) = shell_mesh(collider) else {
            return;
        };
        if let Err(err) = self
            .doc
            .local()
            .set(prim, Property::Transform(hidden()))
            .flush()
        {
            eprintln!("physgun: hide outline: {err:?}");
        }

        match find_one(&self.doc, TEMPLATE_PRIM_NAME) {
            Ok(template) => {
                if let Err(err) = self
                    .doc
                    .local()
                    .set(
                        prim,
                        Property::Relation((Relation::ShaderBinding, template)),
                    )
                    .flush()
                {
                    eprintln!("physgun: bind outline material: {err:?}");
                }
                // The glow graph multiplies its output by this tint;
                // brightness lives in the graph's own intensity input, so
                // alpha is always 1.
                let tint = Color { a: 1.0, ..color };
                if let Err(err) = set_overrides(
                    &self.doc,
                    prim,
                    Layer::Local,
                    &[(TINT_INPUT, GraphValue::Color(tint))],
                ) {
                    eprintln!("physgun: set_overrides: {err:?}");
                }
            }
            Err(err) => {
                eprintln!(
                    "physgun: HSD missing {TEMPLATE_PRIM_NAME} prim; prop unhighlighted: {err:?}"
                );
            }
        }

        *self.prim.borrow_mut() = Some(prim);
    }

    /// Matches the prop's pose at render rate; a shell lagging a frame behind
    /// reads as sliding off the object.
    pub fn track(&self, body: &Transform) {
        let Some(prim) = *self.prim.borrow() else {
            return;
        };
        if let Err(err) = self
            .doc
            .local()
            .set(
                prim,
                Property::Transform(Transform {
                    translation: body.translation,
                    rotation:    body.rotation,
                    scale:       Vec3::ONE,
                }),
            )
            .flush()
        {
            eprintln!("physgun: track outline: {err:?}");
        }
    }

    /// Removes the shell prim outright rather than hiding it: its mesh only
    /// fits the prop it was minted for, and the next grab mints its own.
    pub fn clear(&self) {
        let Some(prim) = self.prim.borrow_mut().take() else {
            return;
        };
        if let Err(err) = self.doc.local().remove(prim).flush() {
            eprintln!("physgun: remove outline: {err:?}");
        }
    }
}
