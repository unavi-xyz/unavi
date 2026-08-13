//! Transcribes the [`SlotView`]s a surface computes into prims.

use std::cell::{
    Cell,
    RefCell,
};

use smol_str::SmolStr;
use wired_prelude::prelude::*;

use crate::{
    attention::Attention,
    mesh,
    mote::{
        Arrange,
        MoteSpec,
        PipPlacement,
    },
    palette::Palette,
    placard::PlacardView,
    scene::{
        draw,
        placard::Placard,
    },
    tree::Mote,
    tuning::Tuning,
    view::{
        SlotView,
        Style,
    },
    wired::{
        input::{
            api::register_input_listener,
            types::{
                InputAction,
                InputListener,
            },
        },
        physics::api::get_linear_velocity,
        scene::types::{
            Collider,
            ColliderCylinder,
            Document,
            Prim,
            RigidBody,
            RigidBodyKind,
            Text,
            TextAlign,
            TextAnchor,
            Xform,
        },
    },
};

/// What a surface's own listener heard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Grab(bool),
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
const SPHERE_RINGS: usize = 10;
const SPHERE_SEGMENTS: usize = 16;
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

struct SlotPrims {
    root:     Prim,
    body:     Prim,
    /// Container children, drawn see-through.
    nested:   Prim,
    /// Leaf children, or depth marks, drawn solid.
    plain:    Prim,
    overflow: Prim,
    /// Whether `overflow` has had its mesh written yet.
    marked:   Cell<bool>,
    /// The mote's name, drawn always rather than on attention.
    label:    Prim,
    /// The mote whose icon is parented here, so a slot reused by another mote
    /// hands the old one back rather than keeping it.
    icon:     RefCell<Option<Mote>>,
    /// Keyed by icon as well as style: an item wears glass only when there is
    /// something inside it to see.
    style:    Cell<Option<(Style, bool)>>,
    shape:    Cell<Option<PipShape>>,
    /// Last `(label, attention)` written; a `set_text` write costs a sync
    /// whether or not the string changed.
    written:  RefCell<Option<(SmolStr, Attention)>>,
}

/// The prims one surface draws with: a hit surface, a pool of slot bodies
/// grown to what is actually shown, and the placard riding whichever of them
/// holds attention.
pub struct Bodies {
    doc:      Document,
    root:     Prim,
    /// The surface's face and its only resting collider. A mote is a drawing
    /// until it is taken, so only this can be hit.
    field:    Prim,
    surface:  Collider,
    /// Grabs against this surface; anything landing elsewhere belongs to
    /// whatever it hit.
    input:    InputListener,
    /// Where an icon waits while the mote holding it is not drawn. Scale zero
    /// rather than unparented: `wired:scene` has no detached prim, and a
    /// removed child would reappear at the document root.
    park:     Prim,
    /// Grown to what the surface is actually drawing, never past `capacity`.
    /// A grid nobody has put anything in costs nothing.
    slots:    RefCell<Vec<SlotPrims>>,
    capacity: usize,
    unit:     mesh::MeshData,
    placard:  Placard,
    /// Whether the surface is up. A dismissed one keeps every prim — the
    /// meshes are paid for once, and a summon that re-uploaded them would
    /// spend a `Flow::BlobUpload` per body every time.
    shown:    Cell<bool>,
}

impl Bodies {
    pub fn new(doc: &Document, capacity: usize, tuning: &Tuning, hit: Hit) -> anyhow::Result<Self> {
        let root = doc.create_prim()?;
        root.set_xform(Some(draw::placed(Vec3::ZERO, 1.0)))?;

        let field = doc.create_prim()?;
        field.set_xform(Some(Xform {
            translation: Vec3::new(0.0, 0.0, FIELD_THICKNESS.mul_add(-0.5, tuning.field_lift)),
            rotation:    hit.rotation(),
            scale:       Vec3::ONE,
        }))?;
        let surface = hit.collider();
        field.set_collider(Some(surface))?;
        root.add_child(&field)?;
        let input = register_input_listener(&field)?;

        let placard = Placard::new(doc, &root)?;

        let park = doc.create_prim()?;
        park.set_xform(Some(draw::hidden()))?;
        root.add_child(&park)?;

        Ok(Self {
            doc: doc.clone(),
            root,
            field,
            surface,
            input,
            park,
            slots: RefCell::new(Vec::with_capacity(capacity)),
            capacity,
            unit: mesh::sphere(1.0, SPHERE_RINGS, SPHERE_SEGMENTS),
            placard,
            shown: Cell::new(true),
        })
    }

    /// Builds up to `count` slots, which is where a surface's geometry cost is
    /// actually paid. A mesh write costs a `BlobUpload` whatever its size, so
    /// this is charged for what is drawn rather than for what might be.
    fn ensure(&self, count: usize) -> anyhow::Result<()> {
        let mut slots = self.slots.borrow_mut();
        for _ in slots.len()..count.min(self.capacity) {
            let slot_root = self.doc.create_prim()?;
            slot_root.set_xform(Some(draw::hidden()))?;
            self.root.add_child(&slot_root)?;

            let body = self.doc.create_prim()?;
            draw::mesh(&body, &self.unit)?;
            body.set_xform(Some(draw::placed(Vec3::ZERO, 1.0)))?;
            slot_root.add_child(&body)?;

            let nested = self.doc.create_prim()?;
            nested.set_xform(Some(draw::hidden()))?;
            slot_root.add_child(&nested)?;

            let plain = self.doc.create_prim()?;
            plain.set_xform(Some(draw::hidden()))?;
            slot_root.add_child(&plain)?;

            // An overflow marker is the exception rather than the rule, so it
            // waits for a slot that has one.
            let overflow = self.doc.create_prim()?;
            overflow.set_xform(Some(draw::hidden()))?;
            slot_root.add_child(&overflow)?;

            // A label rides the slot, not the body, which scales with
            // attention.
            let label = self.doc.create_prim()?;
            label.set_xform(Some(draw::hidden()))?;
            slot_root.add_child(&label)?;

            slots.push(SlotPrims {
                root: slot_root,
                body,
                nested,
                plain,
                overflow,
                marked: Cell::new(false),
                label,
                icon: RefCell::new(None),
                style: Cell::new(None),
                shape: Cell::new(None),
                written: RefCell::new(None),
            });
        }
        Ok(())
    }

    pub fn place(&self, transform: &Transform) -> anyhow::Result<()> {
        self.root.set_xform(Some(Xform {
            translation: transform.translation,
            rotation:    transform.rotation,
            scale:       transform.scale,
        }))?;
        Ok(())
    }

    #[must_use]
    pub const fn root(&self) -> &Prim {
        &self.root
    }

    /// Puts the surface up or takes it down. A surface that is down is not a
    /// wall you cannot see: its collider goes with it.
    pub fn show(&self, shown: bool) -> anyhow::Result<()> {
        if self.shown.replace(shown) == shown {
            return Ok(());
        }
        self.field.set_collider(shown.then_some(self.surface))?;
        if !shown {
            self.root.set_xform(Some(draw::hidden()))?;
        }
        Ok(())
    }

    pub fn poll(&self) -> Vec<Signal> {
        let mut signals = Vec::new();
        while let Some(event) = self.input.poll() {
            match event.action {
                InputAction::GrabDown => signals.push(Signal::Grab(true)),
                InputAction::GrabUp => signals.push(Signal::Grab(false)),
                InputAction::ScrollUp => signals.push(Signal::Turn(-1)),
                InputAction::ScrollDown => signals.push(Signal::Turn(1)),
                _ => {}
            }
        }
        signals
    }

    /// Turns a mote into a dynamic body the engine's grab can take. The
    /// surface's collider stands down while anything is held, so it cannot
    /// intercept the grab's ray.
    pub fn make_dynamic(&self, slot: usize, radius: f32) -> anyhow::Result<()> {
        let slots = self.slots.borrow();
        let Some(prims) = slots.get(slot) else {
            return Ok(());
        };
        prims.root.set_collider(Some(Collider::Sphere(radius)))?;
        // Weightless from promotion: the engine zeroes gravity only once its
        // grab takes the mote, and nothing else holds it up in the gap.
        prims.root.set_gravity_scale(0.0)?;
        prims.root.set_rigid_body(Some(RigidBody {
            kind:            RigidBodyKind::Dynamic,
            angular_damping: None,
            friction:        None,
            linear_damping:  None,
            mass:            None,
            restitution:     None,
        }))?;
        self.field.set_collider(None)?;
        Ok(())
    }

    pub fn clear_dynamic(&self, slot: usize) -> anyhow::Result<()> {
        if let Some(prims) = self.slots.borrow().get(slot) {
            prims.root.set_rigid_body(None)?;
            prims.root.set_collider(None)?;
            // Restored so the next promotion is a real attribute change: the
            // engine resets gravity when it lets go, and an unchanged value
            // would never re-apply ours.
            prims.root.set_gravity_scale(1.0)?;
        }
        self.field
            .set_collider(self.shown.get().then_some(self.surface))?;
        Ok(())
    }

    #[must_use]
    pub fn pose(&self, slot: usize) -> Option<Transform> {
        self.slots
            .borrow()
            .get(slot)
            .map(|prims| prims.root.global_xform())
    }

    #[must_use]
    pub fn velocity(&self, slot: usize) -> Vec3 {
        self.slots
            .borrow()
            .get(slot)
            .and_then(|prims| get_linear_velocity(&prims.root).ok())
            .unwrap_or(Vec3::ZERO)
    }

    /// Puts each drawn mote's icon inside its shell and parks the rest.
    ///
    /// Runs before [`Bodies::apply`], which reads back what a slot ended up
    /// holding to decide whether its shell is glass.
    pub fn icons(&self, motes: &[Mote], views: &[SlotView], drawn: &[usize]) -> anyhow::Result<()> {
        self.ensure(views.len())?;
        let slots = self.slots.borrow();

        let wanted = |slot: usize| {
            drawn
                .get(slot)
                .and_then(|index| motes.get(*index))
                .filter(|mote| slot < views.len() && mote.has_icon())
        };

        // Handing every stale icon back first, so a mote that moved between
        // slots is not parked by the slot it left after arriving.
        for (index, slot) in slots.iter().enumerate() {
            let held = slot.icon.borrow();
            let keep = wanted(index).is_some_and(|mote| held.as_ref().is_some_and(|h| h.is(mote)));
            if keep || held.is_none() {
                continue;
            }
            if let Some(result) = held
                .as_ref()
                .and_then(|h| h.with_icon(|p| self.park.add_child(p)))
            {
                result?;
            }
            drop(held);
            *slot.icon.borrow_mut() = None;
        }

        for (index, slot) in slots.iter().enumerate() {
            let Some(mote) = wanted(index) else {
                continue;
            };
            let radius = views.get(index).map_or(0.0, |view| view.radius);
            if slot.icon.borrow().is_none() {
                if let Some(result) = mote.with_icon(|prim| slot.root.add_child(prim)) {
                    result?;
                }
                *slot.icon.borrow_mut() = Some(mote.clone());
            }
            if let Some(result) =
                mote.with_icon(|prim| prim.set_xform(Some(draw::placed(Vec3::ZERO, radius))))
            {
                result?;
            }
        }
        Ok(())
    }

    /// `drawn` maps each view back to its spec, which pagination makes a real
    /// translation rather than an identity. `held` is the slot the engine is
    /// carrying, whose transform belongs to the solver rather than to us.
    pub fn apply(
        &self,
        views: &[SlotView],
        specs: &[MoteSpec],
        drawn: &[usize],
        placard: Option<&PlacardView>,
        palette: &Palette,
        held: Option<usize>,
    ) -> anyhow::Result<()> {
        match placard {
            Some(view) => self.placard.apply(view, palette)?,
            None => self.placard.hide()?,
        }

        self.ensure(views.len())?;
        let slots = self.slots.borrow();

        for (index, (slot, view)) in slots.iter().zip(views).enumerate() {
            if held != Some(index) {
                slot.root
                    .set_xform(Some(draw::placed(view.position, 1.0)))?;
            }
            slot.body
                .set_xform(Some(draw::placed(Vec3::ZERO, view.radius)))?;

            if let Some(spec) = drawn.get(index).and_then(|index| specs.get(*index)) {
                Self::apply_label(slot, view, spec, palette)?;
            }

            let icon = slot.icon.borrow().is_some();
            if slot.style.get() != Some((view.style, icon)) {
                slot.style.set(Some((view.style, icon)));
                slot.body
                    .set_material(Some(draw::body(view.style, view.role, icon)))?;
                slot.nested
                    .set_material(Some(draw::pip(view.style, true)))?;
                slot.plain
                    .set_material(Some(draw::pip(view.style, false)))?;
                slot.overflow
                    .set_material(Some(draw::pip(view.style, false)))?;
            }

            Self::apply_pips(slot, view)?;
        }

        for slot in slots.iter().skip(views.len()) {
            slot.root.set_xform(Some(draw::hidden()))?;
        }
        Ok(())
    }

    /// Drawn always; only the attended name's brightness changes with
    /// attention.
    fn apply_label(
        slot: &SlotPrims,
        view: &SlotView,
        spec: &MoteSpec,
        palette: &Palette,
    ) -> anyhow::Result<()> {
        slot.label
            .set_xform(Some(draw::placed(view.label_offset, 1.0)))?;

        let key = (spec.label.clone(), view.attention);
        let mut written = slot.written.borrow_mut();
        if written.as_ref() == Some(&key) {
            return Ok(());
        }
        let color = palette.tint(view.attention);
        *written = Some(key);

        slot.label.set_text(Some(&Text {
            value:         spec.label.to_string(),
            size:          Some(view.label_size),
            align:         Some(TextAlign::Center),
            anchor:        Some(TextAnchor::Top),
            wrap:          None,
            line_height:   None,
            color:         Some(color),
            // A surface does not control its surroundings, so its names carry
            // their own contrast.
            outline:       Some(Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.85,
            }),
            outline_width: Some(0.22),
            emissive:      Some(if view.attention.is_active() { 0.5 } else { 0.1 }),
            billboard:     None,
        }))?;
        Ok(())
    }

    /// Drawn unconditionally: how much a container holds is structural.
    fn apply_pips(slot: &SlotPrims, view: &SlotView) -> anyhow::Result<()> {
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

        if slot.shape.get() != Some(shape) {
            slot.shape.set(Some(shape));
            apply_run(&slot.nested, 0, groups, total, arrange, spread, radius)?;
            apply_run(
                &slot.plain,
                groups,
                total - groups,
                total,
                arrange,
                spread,
                radius,
            )?;
        }

        if view.pips.overflow && !slot.marked.get() {
            slot.marked.set(true);
            draw::mesh(&slot.overflow, &mesh::overflow_marker(OVERFLOW_RADIUS))?;
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
            (&slot.nested, (groups > 0).then_some(visible)),
            (&slot.plain, (view.pips.count > groups).then_some(visible)),
            (&slot.overflow, marker),
        ] {
            prim.set_xform(Some(placed.unwrap_or_else(draw::hidden)))?;
        }
        Ok(())
    }
}

fn apply_run(
    prim: &Prim,
    start: usize,
    len: usize,
    total: usize,
    arrange: Arrange,
    spread: f32,
    radius: f32,
) -> anyhow::Result<()> {
    if len == 0 {
        return Ok(());
    }
    draw::mesh(
        prim,
        &mesh::cluster(start, len, total, arrange, spread, radius),
    )
}
