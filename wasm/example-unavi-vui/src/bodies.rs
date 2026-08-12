//! Transcribes [`SlotView`]s computed by `unavi-vui` into prims.

use std::cell::{
    Cell,
    RefCell,
};

use smol_str::SmolStr;
use unavi_vui::{
    attention::Attention,
    mesh,
    mote::{
        MoteSpec,
        PipPlacement,
        Role,
    },
    palette::Palette,
    placard::PlacardView,
    tuning::Tuning,
    view::{
        SlotView,
        Style,
    },
};
use wired_prelude::prelude::*;

use crate::{
    placard::Placard,
    wired::{
        input::{
            api::register_input_listener,
            types::{
                InputAction,
                InputListener,
            },
        },
        physics::api::get_linear_velocity,
        scene::api::self_document,
        scene::types::{
            AlphaMode,
            Collider,
            ColliderCylinder,
            Document,
            Material,
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

/// What the surface's own listener heard.
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
    /// A rack's shelf front.
    Slab { extents: Vec2 },
}

impl Hit {
    fn collider(self) -> Collider {
        match self {
            Self::Disc { radius } => Collider::Cylinder(ColliderCylinder {
                height: FIELD_THICKNESS,
                radius,
            }),
            Self::Slab { extents } => Collider::Cuboid(Vec3::new(
                extents.x * 2.0,
                extents.y * 2.0,
                FIELD_THICKNESS,
            )),
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
/// Thin enough to read as the dial's own face, thick enough to be a solid
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
    style:    Cell<Option<Style>>,
    shape:    Cell<Option<PipShape>>,
    /// Last `(label, attention)` written; a `set_text` write costs a sync
    /// whether or not the string changed.
    written:  RefCell<Option<(SmolStr, Attention)>>,
}

pub struct Bodies {
    doc:      Document,
    root:     Prim,
    /// The dial's surface and the orbit's only resting collider. A mote is a
    /// drawing until it is taken, so only this can be hit.
    field:    Prim,
    surface:  Collider,
    /// Grabs against the dial's surface; anything landing elsewhere belongs
    /// to whatever it hit.
    input:    InputListener,
    /// Grown to what the surface is actually drawing, never past `capacity`.
    /// A rack nobody has put anything in costs nothing.
    slots:    RefCell<Vec<SlotPrims>>,
    capacity: usize,
    unit:     mesh::MeshData,
    placard:  Placard,
}

impl Bodies {
    pub fn new(doc: &Document, capacity: usize, tuning: &Tuning, hit: Hit) -> anyhow::Result<Self> {
        let root = doc.create_prim()?;
        root.set_xform(Some(placed(Vec3::ZERO, 1.0)))?;

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

        Ok(Self {
            doc: self_document()?,
            root,
            field,
            surface,
            input,
            slots: RefCell::new(Vec::with_capacity(capacity)),
            capacity,
            unit: mesh::sphere(1.0, SPHERE_RINGS, SPHERE_SEGMENTS),
            placard,
        })
    }

    /// Builds up to `count` slots, which is where a surface's geometry cost is
    /// actually paid. A mesh write costs a `BlobUpload` whatever its size, so
    /// this is charged for what is drawn rather than for what might be.
    fn ensure(&self, count: usize) -> anyhow::Result<()> {
        let mut slots = self.slots.borrow_mut();
        for _ in slots.len()..count.min(self.capacity) {
            let slot_root = self.doc.create_prim()?;
            slot_root.set_xform(Some(hidden()))?;
            self.root.add_child(&slot_root)?;

            let body = self.doc.create_prim()?;
            apply_mesh(&body, &self.unit)?;
            body.set_xform(Some(placed(Vec3::ZERO, 1.0)))?;
            slot_root.add_child(&body)?;

            let nested = self.doc.create_prim()?;
            nested.set_xform(Some(hidden()))?;
            slot_root.add_child(&nested)?;

            let plain = self.doc.create_prim()?;
            plain.set_xform(Some(hidden()))?;
            slot_root.add_child(&plain)?;

            // An overflow marker is the exception rather than the rule, so it
            // waits for a slot that has one.
            let overflow = self.doc.create_prim()?;
            overflow.set_xform(Some(hidden()))?;
            slot_root.add_child(&overflow)?;

            // A label rides the slot, not the body, which scales with
            // attention.
            let label = self.doc.create_prim()?;
            label.set_xform(Some(hidden()))?;
            slot_root.add_child(&label)?;

            slots.push(SlotPrims {
                root: slot_root,
                body,
                nested,
                plain,
                overflow,
                marked: Cell::new(false),
                label,
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
    /// field's collider stands down while anything is held, so it cannot
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
        self.field.set_collider(Some(self.surface))?;
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
                slot.root.set_xform(Some(placed(view.position, 1.0)))?;
            }
            slot.body.set_xform(Some(placed(Vec3::ZERO, view.radius)))?;

            if let Some(spec) = drawn.get(index).and_then(|index| specs.get(*index)) {
                Self::apply_label(slot, view, spec, palette)?;
            }

            if slot.style.get() != Some(view.style) {
                slot.style.set(Some(view.style));
                slot.body.set_material(Some(shell(view.style, view.role)))?;
                slot.nested
                    .set_material(Some(pip_material(view.style, true)))?;
                slot.plain
                    .set_material(Some(pip_material(view.style, false)))?;
                slot.overflow
                    .set_material(Some(pip_material(view.style, false)))?;
            }

            Self::apply_pips(slot, view)?;
        }

        for slot in slots.iter().skip(views.len()) {
            slot.root.set_xform(Some(hidden()))?;
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
        slot.label.set_xform(Some(placed(view.label_offset, 1.0)))?;

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
            // A dial does not control its surroundings, so its names carry
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
        };

        if slot.shape.get() != Some(shape) {
            slot.shape.set(Some(shape));
            let (ring, radius) = match view.pips.placement {
                PipPlacement::Inside => (INSIDE_ORBIT, PIP_RADIUS),
                PipPlacement::Around => (AROUND_ORBIT, MARK_RADIUS),
            };
            let total = view.pips.count;
            apply_run(&slot.nested, 0, groups, total, ring, radius)?;
            apply_run(&slot.plain, groups, total - groups, total, ring, radius)?;
        }

        if view.pips.overflow && !slot.marked.get() {
            slot.marked.set(true);
            apply_mesh(&slot.overflow, &mesh::overflow_marker(OVERFLOW_RADIUS))?;
        }

        let visible = placed(Vec3::ZERO, view.radius);
        for (prim, shown) in [
            (&slot.nested, groups > 0),
            (&slot.plain, view.pips.count > groups),
            (&slot.overflow, view.pips.overflow),
        ] {
            prim.set_xform(Some(if shown { visible } else { hidden() }))?;
        }
        Ok(())
    }
}

fn apply_run(
    prim: &Prim,
    start: usize,
    len: usize,
    total: usize,
    ring: f32,
    radius: f32,
) -> anyhow::Result<()> {
    if len == 0 {
        return Ok(());
    }
    apply_mesh(prim, &mesh::cluster(start, len, total, ring, radius))
}

fn apply_mesh(prim: &Prim, data: &mesh::MeshData) -> anyhow::Result<()> {
    prim.set_mesh_stream("POSITION", Some(&data.positions))?;
    prim.set_mesh_stream("NORMAL", Some(&data.normals))?;
    prim.set_mesh_indices_u32(Some(&data.indices))?;
    Ok(())
}

const fn placed(translation: Vec3, scale: f32) -> Xform {
    Xform {
        translation,
        rotation: Quat::IDENTITY,
        scale: Vec3::splat(scale),
    }
}

const fn hidden() -> Xform {
    placed(Vec3::ZERO, 0.0)
}

const fn with_alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a,
    }
}

const fn scaled(color: Color, factor: f32) -> Color {
    Color {
        r: color.r * factor,
        g: color.g * factor,
        b: color.b * factor,
        a: 1.0,
    }
}

const fn shell(style: Style, role: Role) -> Material {
    let opaque = matches!(role, Role::Action | Role::Cast | Role::Parent { .. });
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(if opaque {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        }),
        base_color:   Some(with_alpha(style.color, style.alpha)),
        double_sided: Some(!opaque),
        emissive:     Some(scaled(style.color, style.emissive)),
        metallic:     None,
        roughness:    None,
    }
}

/// A container pip is see-through, like the mote it stands for.
const fn pip_material(style: Style, nested: bool) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(if nested {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        }),
        base_color:   Some(with_alpha(style.color, if nested { 0.35 } else { 1.0 })),
        double_sided: Some(nested),
        emissive:     Some(scaled(style.color, style.emissive * 1.6)),
        metallic:     None,
        roughness:    None,
    }
}
