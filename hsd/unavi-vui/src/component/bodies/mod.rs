//! Transcribes the [`SlotView`]s a surface computes into prims.
//!
//! Split across this module's files by concern: [`mod@apply`] writes a
//! drawing into prims, [`mod@dock`] parks and fits icons, [`mod@face`] draws
//! and flips glyphs, [`mod@hitbox`] reads the surface's own input listener,
//! and [`mod@prims`] holds what one slot owns. Each adds `impl Bodies` blocks
//! to the one type defined here.

use std::{
    cell::{
        Cell,
        RefCell,
    },
    rc::Rc,
};

use wired_guest::math::{
    Quat,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    component::{
        bodies::prims::{
            Mark,
            Marks,
            SlotPrims,
        },
        placard::{
            PlacardView,
            prims::PlacardPrims,
        },
        tree::Mote,
    },
    face::marks::Chevrons,
    machinery::fit::chain,
    material::{
        glyph::Glyph,
        graphs,
    },
    part::{
        draw,
        mesh,
        shadow::Shadow,
        spec::MoteSpec,
        view::SlotView,
    },
    tokens::{
        palette::Palette,
        tuning::Tuning,
    },
    wired::{
        core::error::Error,
        input::{
            targeted::listen,
            types::InputSubscription,
        },
        physics::simulation::velocity as physics_velocity,
        scene::{
            document::{
                Document,
                Layer,
            },
            properties::{
                Collider,
                ColliderCylinder,
                Property,
                PropertyKey,
                RigidBody,
                RigidBodyKind,
            },
        },
    },
};

mod apply;
mod dock;
mod face;
mod hitbox;
mod prims;

use face::{
    Face,
    Shape,
};

/// What a surface's own listener heard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    /// The trigger: acting on the lit mote where it is.
    Act(bool),
    /// The grip: taking hold of the lit mote, so travelling with it before
    /// letting go puts it down somewhere new rather than firing it.
    Take(bool),
    /// Pages, in the direction of the scroll.
    Turn(isize),
}

/// The one resting collider a surface carries. It covers the region its layout
/// answers for and the page tabs standing outside it.
#[derive(Debug, Clone, Copy)]
pub enum Hit {
    /// An orbit's dial face.
    Disc { radius: f32 },
    /// A grid's front.
    Slab { extents: Vec2 },
}

impl Hit {
    fn collider(self) -> Collider {
        match self {
            Self::Disc { radius } => Collider::Cylinder(ColliderCylinder {
                height: FIELD_THICKNESS,
                radius,
            }),
            Self::Slab { extents } => {
                Collider::Cuboid(Vec3::new(extents.x * 2.0, extents.y * 2.0, FIELD_THICKNESS))
            }
        }
    }

    /// A cylinder stands on its own Y, and a surface faces along Z.
    fn rotation(self) -> Quat {
        match self {
            Self::Disc { .. } => Quat::new(0.5_f32.sqrt(), 0.0, 0.0, 0.5_f32.sqrt()),
            Self::Slab { .. } => Quat::IDENTITY,
        }
    }
}

/// A mote is a bubble, and a bubble's silhouette is the thing being read, so
/// it is worth the vertices to have no visible facets on it. Paid once per
/// slot, at one blob upload per stream whatever the size.
const SPHERE_RINGS: usize = 28;
const SPHERE_SEGMENTS: usize = 40;
/// Thin enough to read as the surface's own face, thick enough to be a solid
/// raycast target.
const FIELD_THICKNESS: f32 = 0.01;

/// Everything a frame's worth of drawing reads, borrowed from the surface.
///
/// `drawn` maps each view back to its spec, which pagination makes a real
/// translation rather than an identity. `held` is the slot the engine is
/// carrying, whose transform belongs to the solver rather than to us. `motes`
/// lines up with `specs`, and says whose face each slot is wearing.
#[derive(Clone, Copy)]
pub struct Drawing<'a> {
    pub motes:         &'a [Mote],
    /// Seconds since the last frame, which a flip runs on.
    pub delta:         f32,
    /// The text drawn at the centre, when the level has an open one. Not a
    /// slot: it has no view, no spec and nothing to attend.
    pub readout:       Option<&'a Rc<Glyph>>,
    /// How far the centre has arrived, which the readout comes in by.
    pub readout_bloom: f32,
    pub views:         &'a [SlotView],
    pub specs:         &'a [MoteSpec],
    pub drawn:         &'a [usize],
    pub placard:       Option<&'a PlacardView>,
    pub palette:       &'a Palette,
    pub held:          Option<usize>,
    /// The viewer's eye in the surface's own space, which a face turns toward.
    pub eye:           Vec3,
}

/// The prims one surface draws with: a hit surface, a pool of slot bodies
/// grown to what is actually shown, and the placard riding whichever of them
/// holds attention.
pub struct Bodies {
    root:     (u64, u64),
    /// The surface's face and its only resting collider. A mote is a drawing
    /// until it is taken, so only this can be hit.
    field:    (u64, u64),
    surface:  Collider,
    /// Grabs against this surface; anything landing elsewhere belongs to
    /// whatever it hit.
    input:    InputSubscription,
    /// Where an icon waits while the mote holding it is not drawn. Scale zero
    /// rather than unparented: `wired:scene` has no detached prim, and a
    /// removed child would reappear at the document root.
    park:     (u64, u64),
    /// A readout's face at the surface's centre, drawn apart from the slots.
    readout:  Face,
    /// What the way back and the page tabs wear.
    chevrons: Chevrons,
    /// Grown to what the surface is actually drawing, never past `capacity`.
    /// A grid nobody has put anything in costs nothing.
    slots:    RefCell<Vec<SlotPrims>>,
    capacity: usize,
    unit:     mesh::MeshData,
    placard:  PlacardPrims,
    tuning:   Tuning,
    /// Radians the drawn icons have turned, accumulated per frame so the
    /// spin is smooth however fast the tick.
    spin:     Cell<f32>,
    /// Whether the surface is up. A dismissed one keeps every prim — the
    /// meshes are paid for once, and a summon that re-uploaded them would
    /// spend a `Flow::BlobUpload` per body every time.
    shown:    Cell<bool>,
}

impl Bodies {
    pub fn new(doc: &Document, capacity: usize, tuning: &Tuning, hit: Hit) -> anyhow::Result<Self> {
        let root = doc.create_prim(Layer::Local, None)?;
        doc.local()
            .set(root, Property::Transform(draw::placed(Vec3::ZERO, 1.0)))
            .flush()?;

        let field = doc.create_prim(Layer::Local, Some(root))?;
        let surface = hit.collider();
        doc.local()
            .set(
                field,
                Property::Transform(Transform {
                    translation: Vec3::new(
                        0.0,
                        0.0,
                        FIELD_THICKNESS.mul_add(-0.5, tuning.field_lift),
                    ),
                    rotation:    hit.rotation(),
                    scale:       Vec3::ONE,
                }),
            )
            .set(field, Property::Collider(surface))
            .flush()?;
        let input = listen(doc, field)?;

        let placard = PlacardPrims::new(doc, root)?;

        let park = doc.create_prim(Layer::Local, Some(root))?;
        doc.local()
            .set(park, Property::Transform(draw::hidden()))
            .flush()?;

        let readout = doc.create_prim(Layer::Local, Some(root))?;
        doc.local()
            .set(readout, Property::Transform(draw::hidden()))
            .flush()?;

        Ok(Self {
            root,
            field,
            surface,
            input,
            park,
            readout: Face::new(readout, Shape::Quad),
            chevrons: Chevrons::new().map_err(|err| anyhow::anyhow!("{err}"))?,
            slots: RefCell::new(Vec::with_capacity(capacity)),
            capacity,
            unit: mesh::sphere(1.0, SPHERE_RINGS, SPHERE_SEGMENTS),
            placard,
            tuning: *tuning,
            spin: Cell::new(0.0),
            shown: Cell::new(true),
        })
    }

    /// Builds up to `count` slots, which is where a surface's geometry cost is
    /// actually paid. A mesh write costs a `BlobUpload` whatever its size, so
    /// this is charged for what is drawn rather than for what might be.
    fn ensure(&self, doc: &Document, count: usize) -> anyhow::Result<()> {
        let mut slots = self.slots.borrow_mut();
        for _ in slots.len()..count.min(self.capacity) {
            let slot_root = doc.create_prim(Layer::Local, Some(self.root))?;
            doc.local()
                .set(slot_root, Property::Transform(draw::hidden()))
                .flush()?;

            let body = doc.create_prim(Layer::Local, Some(slot_root))?;
            let batch = draw::mesh(&self.unit)
                .into_iter()
                .fold(doc.local(), |batch, property| batch.set(body, property));
            batch
                .set(body, Property::Transform(draw::placed(Vec3::ZERO, 1.0)))
                .flush()?;
            graphs::set_shell(doc, body)?;

            let nested = doc.create_prim(Layer::Local, Some(slot_root))?;
            let plain = doc.create_prim(Layer::Local, Some(slot_root))?;
            // An overflow marker is the exception rather than the rule, so it
            // waits for a slot that has one.
            let overflow = doc.create_prim(Layer::Local, Some(slot_root))?;
            // A label rides the slot, not the body, which scales with
            // attention.
            let label = doc.create_prim(Layer::Local, Some(slot_root))?;
            let bar = doc.create_prim(Layer::Local, Some(slot_root))?;
            let ring = doc.create_prim(Layer::Local, Some(slot_root))?;
            let face = doc.create_prim(Layer::Local, Some(slot_root))?;
            doc.local()
                .set(face, Property::Transform(draw::hidden()))
                .set(bar, Property::Transform(draw::hidden()))
                .set(ring, Property::Transform(draw::hidden()))
                .flush()?;
            doc.local()
                .set(nested, Property::Transform(draw::hidden()))
                .set(plain, Property::Transform(draw::hidden()))
                .set(overflow, Property::Transform(draw::hidden()))
                .set(label, Property::Transform(draw::hidden()))
                .flush()?;

            slots.push(SlotPrims {
                root: slot_root,
                root_xform: Shadow::new(),
                body,
                body_xform: Shadow::new(),
                nested,
                plain,
                overflow,
                marked: Cell::new(false),
                label,
                marks: Marks {
                    bar:  Mark::new(bar),
                    ring: Mark::new(ring),
                },
                face: Face::new(
                    face,
                    Shape::Dome {
                        fill:  self.tuning.icon_fill,
                        reach: self.tuning.icon_fill * 1.05,
                    },
                ),
                label_xform: Shadow::new(),
                icon: RefCell::new(None),
                shell: Shadow::new(),
                shape: Shadow::new(),
                written: Shadow::new(),
            });
        }
        Ok(())
    }

    pub fn place(&self, doc: &Document, transform: &Transform) -> anyhow::Result<()> {
        doc.local()
            .set(
                self.root,
                Property::Transform(Transform {
                    translation: transform.translation,
                    rotation:    transform.rotation,
                    scale:       transform.scale,
                }),
            )
            .flush()?;
        Ok(())
    }

    #[must_use]
    pub const fn root(&self) -> (u64, u64) {
        self.root
    }

    /// Puts the surface up or takes it down.
    ///
    /// The collider goes the moment it is sent away rather than when it
    /// finishes leaving: a surface on its way out is not a wall you cannot
    /// see, and a press landing on one that is already going is a press
    /// nobody meant. The bodies keep animating out on their own.
    pub fn show(&self, doc: &Document, shown: bool) -> anyhow::Result<()> {
        if self.shown.replace(shown) == shown {
            return Ok(());
        }
        let mut batch = doc.local();
        batch = if shown {
            batch.set(self.field, Property::Collider(self.surface))
        } else {
            batch.clear(self.field, PropertyKey::Collider)
        };
        batch.flush()?;
        Ok(())
    }

    /// Turns a mote into a dynamic body the engine's grab can take. The
    /// surface's collider stands down while anything is held, so it cannot
    /// intercept the grab's ray.
    pub fn make_dynamic(&self, doc: &Document, slot: usize, radius: f32) -> anyhow::Result<()> {
        let slots = self.slots.borrow();
        let Some(prims) = slots.get(slot) else {
            return Ok(());
        };
        doc.local()
            .set(prims.root, Property::Collider(Collider::Sphere(radius)))
            // Weightless from promotion: the engine zeroes gravity only once
            // its grab takes the mote, and nothing else holds it up in the
            // gap.
            .set(prims.root, Property::GravityScale(0.0))
            .set(
                prims.root,
                Property::RigidBody(RigidBody {
                    kind:            RigidBodyKind::Dynamic,
                    angular_damping: None,
                    friction:        None,
                    linear_damping:  None,
                    mass:            None,
                    restitution:     None,
                }),
            )
            .clear(self.field, PropertyKey::Collider)
            .flush()?;
        Ok(())
    }

    pub fn clear_dynamic(&self, doc: &Document, slot: usize) -> anyhow::Result<()> {
        let mut batch = doc.local();
        if let Some(prims) = self.slots.borrow().get(slot) {
            batch = batch
                .clear(prims.root, PropertyKey::RigidBody)
                .clear(prims.root, PropertyKey::Collider)
                // Restored so the next promotion is a real attribute change:
                // the engine resets gravity when it lets go, and an unchanged
                // value would never re-apply ours.
                .set(prims.root, Property::GravityScale(1.0));
        }
        batch = if self.shown.get() {
            batch.set(self.field, Property::Collider(self.surface))
        } else {
            batch.clear(self.field, PropertyKey::Collider)
        };
        batch.flush()?;
        Ok(())
    }

    /// Where `slot` stands. One no frame has drawn yet stands where it was
    /// last written, which is where it is about to be drawn.
    #[must_use]
    pub fn pose(&self, doc: &Document, slot: usize) -> Option<Transform> {
        let prim = self.slots.borrow().get(slot)?.root;
        match doc.world_transform(prim) {
            Ok(pose) => Some(pose),
            Err(Error::NotReady) => Some(chain(&stated(doc, self.root)?, &stated(doc, prim)?)),
            Err(_) => None,
        }
    }

    #[must_use]
    pub fn velocity(&self, doc: &Document, slot: usize) -> Vec3 {
        self.slots
            .borrow()
            .get(slot)
            .and_then(|prims| physics_velocity(doc, prims.root).ok())
            .map_or(Vec3::ZERO, |velocity| velocity.linear)
    }
}

/// The transform `prim` was written with, beneath its parent.
fn stated(doc: &Document, prim: (u64, u64)) -> Option<Transform> {
    match doc.get(prim, &PropertyKey::Transform)? {
        Property::Transform(transform) => Some(transform),
        _ => None,
    }
}
