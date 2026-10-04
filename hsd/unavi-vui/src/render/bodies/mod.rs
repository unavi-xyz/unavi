//! Transcribes the [`SlotView`]s a surface computes into prims.
//!
//! Split across this module's files by concern: [`mod@dock`] parks and fits
//! icons, [`mod@hitbox`] reads the surface's own input listener. Both add
//! `impl Bodies` blocks to the one type defined here.

use std::cell::{
    Cell,
    RefCell,
};

use smol_str::SmolStr;
use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    attention::Attention,
    mesh,
    mote::{
        Arrange,
        MoteSpec,
        PipPlacement,
        Role,
    },
    palette::Palette,
    placard::PlacardView,
    render::{
        draw,
        graphs,
        placard::PlacardPrims,
        shadow::Shadow,
    },
    tree::Mote,
    tuning::Tuning,
    view::{
        SlotView,
        Style,
    },
    wired::{
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
                Text,
                TextAlign,
                TextAnchor,
            },
        },
        shading::graph::{
            GraphValue,
            set_overrides,
        },
    },
};

mod dock;
mod hitbox;

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

/// The one resting collider a surface carries, and exactly the region its
/// layout answers for.
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

/// Contents sit within the body; depth marks ride outside it.
const INSIDE_ORBIT: f32 = 0.52;
const AROUND_ORBIT: f32 = 1.35;
const PIP_RADIUS: f32 = 0.13;
const MARK_RADIUS: f32 = 0.09;
const OVERFLOW_RADIUS: f32 = 0.11;
/// A mote is a bubble, and a bubble's silhouette is the thing being read, so
/// it is worth the vertices to have no visible facets on it. Paid once per
/// slot, at one blob upload per stream whatever the size.
const SPHERE_RINGS: usize = 28;
const SPHERE_SEGMENTS: usize = 40;
/// Thin enough to read as the surface's own face, thick enough to be a solid
/// raycast target.
const FIELD_THICKNESS: f32 = 0.01;

/// Last pip-mesh inputs. Rebuilding costs blob uploads, so nothing is
/// re-uploaded while this is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PipShape {
    count:     usize,
    groups:    usize,
    overflow:  bool,
    placement: PipPlacement,
    arrange:   Arrange,
}

/// Everything the shell graph is handed. Not derivable from the style alone:
/// an item wears glass only when there is something inside it to see, and the
/// film and frost are per-mote choices.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Shell {
    style: Style,
    icon:  bool,
    film:  f32,
    frost: f32,
    bloom: f32,
}

struct SlotPrims {
    root:        (u64, u64),
    root_xform:  Shadow<Transform>,
    body:        (u64, u64),
    body_xform:  Shadow<Transform>,
    /// Container children, drawn see-through.
    nested:      (u64, u64),
    /// Leaf children, or depth marks, drawn solid.
    plain:       (u64, u64),
    overflow:    (u64, u64),
    /// Whether `overflow` has had its mesh written yet.
    marked:      Cell<bool>,
    /// The way back's own mark, which no other role wears.
    back:        (u64, u64),
    /// Whether `back` has had its mesh written yet.
    carved:      Cell<bool>,
    /// The mote's name, drawn always rather than on attention.
    label:       (u64, u64),
    label_xform: Shadow<Vec3>,
    /// The mote whose icon is parented here, so a slot reused by another mote
    /// hands the old one back rather than keeping it.
    icon:        RefCell<Option<Mote>>,
    /// What the shell graph was last handed, so a settled mote is not written
    /// every frame.
    shell:       Shadow<Shell>,
    shape:       Shadow<PipShape>,
    /// Last `(label, attention)` written; a `set_text` write costs a sync
    /// whether or not the string changed.
    written:     Shadow<(SmolStr, Attention)>,
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

        Ok(Self {
            root,
            field,
            surface,
            input,
            park,
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
            // Likewise the way back, which at most one slot of a level is.
            let back = doc.create_prim(Layer::Local, Some(slot_root))?;
            // A label rides the slot, not the body, which scales with
            // attention.
            let label = doc.create_prim(Layer::Local, Some(slot_root))?;
            doc.local()
                .set(nested, Property::Transform(draw::hidden()))
                .set(plain, Property::Transform(draw::hidden()))
                .set(overflow, Property::Transform(draw::hidden()))
                .set(back, Property::Transform(draw::hidden()))
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
                back,
                carved: Cell::new(false),
                label,
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

    #[must_use]
    pub fn pose(&self, doc: &Document, slot: usize) -> Option<Transform> {
        let prim = self.slots.borrow().get(slot)?.root;
        doc.world_transform(prim).ok()
    }

    #[must_use]
    pub fn velocity(&self, doc: &Document, slot: usize) -> Vec3 {
        self.slots
            .borrow()
            .get(slot)
            .and_then(|prims| physics_velocity(doc, prims.root).ok())
            .map_or(Vec3::ZERO, |velocity| velocity.linear)
    }

    /// `drawn` maps each view back to its spec, which pagination makes a real
    /// translation rather than an identity. `held` is the slot the engine is
    /// carrying, whose transform belongs to the solver rather than to us.
    #[expect(clippy::too_many_arguments)]
    pub fn apply(
        &self,
        doc: &Document,
        views: &[SlotView],
        specs: &[MoteSpec],
        drawn: &[usize],
        placard: Option<&PlacardView>,
        palette: &Palette,
        held: Option<usize>,
    ) -> anyhow::Result<()> {
        match placard {
            Some(view) => self.placard.apply(doc, view, palette)?,
            None => self.placard.hide(doc)?,
        }

        self.ensure(doc, views.len())?;
        let slots = self.slots.borrow();

        for (index, (slot, view)) in slots.iter().zip(views).enumerate() {
            let mut batch = doc.local();
            if held == Some(index) {
                // The solver owns this slot's transform until it comes back;
                // forgetting what we last wrote makes the first frame back
                // write for sure, rather than trusting it matches.
                slot.root_xform.clear();
            } else if let Some(xform) = slot
                .root_xform
                .diff(draw::placed(view.position, view.bloom))
            {
                batch = batch.set(slot.root, Property::Transform(xform));
            }
            if let Some(xform) = slot.body_xform.diff(draw::placed(Vec3::ZERO, view.radius)) {
                batch = batch.set(slot.body, Property::Transform(xform));
            }

            let spec = drawn.get(index).and_then(|index| specs.get(*index));
            if let Some(spec) = spec {
                batch = Self::apply_label(batch, slot, view, spec, palette);
            }

            let shell = Shell {
                style: view.style,
                icon:  slot.icon.borrow().is_some(),
                film:  spec.map_or(0.0, |spec| spec.film),
                frost: spec.map_or(0.0, |spec| spec.frost),
                bloom: view.bloom,
            };
            // Not `Option::map_or`: the closure would need to move `batch`
            // out of this scope, and a `Batch` is not `Clone`.
            #[expect(clippy::option_if_let_else)]
            let overrides = if let Some(shell) = slot.shell.diff(shell) {
                batch = batch
                    .set(slot.nested, Property::Material(draw::pip(view.style, true)))
                    .set(slot.plain, Property::Material(draw::pip(view.style, false)))
                    .set(
                        slot.overflow,
                        Property::Material(draw::pip(view.style, false)),
                    )
                    .set(slot.back, Property::Material(draw::pip(view.style, false)));
                Some((index, shell))
            } else {
                None
            };

            batch = Self::apply_pips(batch, slot, view);
            batch = Self::apply_back(batch, slot, view);
            batch.flush()?;

            if let Some((index, shell)) = overrides {
                Self::apply_shell(doc, slot, view, index, shell, palette)?;
            }
        }

        let mut batch = doc.local();
        for slot in slots.iter().skip(views.len()) {
            batch = batch.set(slot.root, Property::Transform(draw::hidden()));
        }
        batch.flush()?;
        Ok(())
    }

    /// Hands the shell graph what differs between one mote and the next.
    ///
    /// The palette still decides colour and the graph decides light: it is
    /// given a finished tint rather than a heat to interpret, so every rule
    /// about what a mote's colour means stays where it is written and tested.
    /// Heat goes over only because a rim is per-fragment and cannot be
    /// computed here.
    fn apply_shell(
        doc: &Document,
        slot: &SlotPrims,
        view: &SlotView,
        index: usize,
        shell: Shell,
        palette: &Palette,
    ) -> anyhow::Result<()> {
        // An item wears glass exactly when there is something inside it to
        // see, which is the one thing about a mote's alpha that depends on
        // what it ended up holding rather than on what it is.
        let alpha = if shell.icon && matches!(view.role, Role::Item { .. }) {
            palette.glass(view.attention)
        } else {
            shell.style.alpha
        };

        set_overrides(
            doc,
            slot.body,
            Layer::Local,
            &[
                (
                    graphs::SHELL_TINT,
                    GraphValue::Color(draw::with_alpha(shell.style.color, alpha)),
                ),
                (
                    graphs::SHELL_EMISSIVE,
                    GraphValue::Float(shell.style.emissive),
                ),
                (graphs::SHELL_HEAT, GraphValue::Float(view.heat)),
                (graphs::SHELL_PHASE, GraphValue::Float(phase(index))),
                (graphs::SHELL_FILM, GraphValue::Float(shell.film)),
                (graphs::SHELL_FROST, GraphValue::Float(shell.frost)),
                (graphs::SHELL_BLOOM, GraphValue::Float(shell.bloom)),
            ],
        )?;
        Ok(())
    }

    /// Drawn always; only the attended name's brightness changes with
    /// attention.
    fn apply_label<'a>(
        mut batch: crate::Batch<'a>,
        slot: &SlotPrims,
        view: &SlotView,
        spec: &MoteSpec,
        palette: &Palette,
    ) -> crate::Batch<'a> {
        if let Some(offset) = slot.label_xform.diff(view.label_offset) {
            batch = batch.set(slot.label, Property::Transform(draw::placed(offset, 1.0)));
        }

        let Some((label, attention)) = slot.written.diff((spec.label.clone(), view.attention))
        else {
            return batch;
        };
        let color = palette.tint(attention);

        batch.set(
            slot.label,
            Property::Text(Text {
                value:         label.to_string(),
                size:          Some(view.label_size),
                align:         Some(TextAlign::Center),
                anchor:        Some(TextAnchor::Top),
                wrap:          None,
                line_height:   None,
                color:         Some(color),
                // A surface does not control its surroundings, so its names
                // carry their own contrast.
                outline:       Some(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.85,
                }),
                outline_width: Some(0.22),
                emissive:      Some(if attention.is_active() { 0.5 } else { 0.1 }),
                billboard:     None,
            }),
        )
    }

    /// The way back wears a mark of its own, so a glance tells it from the
    /// level it climbs out of.
    fn apply_back<'a>(
        mut batch: crate::Batch<'a>,
        slot: &SlotPrims,
        view: &SlotView,
    ) -> crate::Batch<'a> {
        let leaving = matches!(view.role, Role::Parent { .. });
        if leaving && !slot.carved.get() {
            slot.carved.set(true);
            batch = draw::mesh(&mesh::chevron())
                .into_iter()
                .fold(batch, |batch, property| batch.set(slot.back, property));
        }
        batch.set(
            slot.back,
            Property::Transform(if leaving {
                draw::placed(Vec3::ZERO, view.radius)
            } else {
                draw::hidden()
            }),
        )
    }

    /// Drawn unconditionally: how much a container holds is structural.
    fn apply_pips<'a>(
        mut batch: crate::Batch<'a>,
        slot: &SlotPrims,
        view: &SlotView,
    ) -> crate::Batch<'a> {
        let groups = view.pips.groups();
        let shape = PipShape {
            count: view.pips.count,
            groups,
            overflow: view.pips.overflow,
            placement: view.pips.placement,
            arrange: view.pips.arrange,
        };

        let arrange = view.pips.arrange;
        let total = view.pips.count;
        let (spread, radius) = match view.pips.placement {
            PipPlacement::Inside => (INSIDE_ORBIT, PIP_RADIUS),
            PipPlacement::Around => (AROUND_ORBIT, MARK_RADIUS),
        };

        if slot.shape.diff(shape).is_some() {
            batch = apply_run(
                batch,
                slot.nested,
                0,
                groups,
                total,
                arrange,
                spread,
                radius,
            );
            batch = apply_run(
                batch,
                slot.plain,
                groups,
                total - groups,
                total,
                arrange,
                spread,
                radius,
            );
        }

        if view.pips.overflow && !slot.marked.get() {
            slot.marked.set(true);
            batch = draw::mesh(&mesh::overflow_marker(OVERFLOW_RADIUS))
                .into_iter()
                .fold(batch, |batch, property| batch.set(slot.overflow, property));
        }

        let visible = draw::placed(Vec3::ZERO, view.radius);
        // The pips are unit-space and scaled by the body; the marker's mesh is
        // centred, so its own offset is scaled the same way by hand. Asked for
        // only when there is a marker: an empty level has no cell after its
        // last pip.
        let marker = view.pips.overflow.then(|| {
            let at = Vec3::from_array(mesh::overflow_at(arrange, total, spread));
            draw::placed(at * view.radius, view.radius)
        });

        for (prim, placed) in [
            (slot.nested, (groups > 0).then_some(visible)),
            (slot.plain, (view.pips.count > groups).then_some(visible)),
            (slot.overflow, marker),
        ] {
            batch = batch.set(
                prim,
                Property::Transform(placed.unwrap_or_else(draw::hidden)),
            );
        }
        batch
    }
}

/// A slot's offset into the idle breath. The golden angle, so neighbours never
/// land near each other and a form reads as several bodies rather than one.
fn phase(slot: usize) -> f32 {
    slot as f32 * 2.399_963
}

#[expect(clippy::too_many_arguments)]
fn apply_run(
    batch: crate::Batch<'_>,
    prim: (u64, u64),
    start: usize,
    len: usize,
    total: usize,
    arrange: Arrange,
    spread: f32,
    radius: f32,
) -> crate::Batch<'_> {
    if len == 0 {
        return batch;
    }
    draw::mesh(&mesh::cluster(start, len, total, arrange, spread, radius))
        .into_iter()
        .fold(batch, |batch, property| batch.set(prim, property))
}
