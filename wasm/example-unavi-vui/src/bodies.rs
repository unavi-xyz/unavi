//! Prim pool. The only thing this file decides is how a [`SlotView`] becomes
//! prims; every value it reads was computed by `unavi-vui`.

use std::cell::Cell;

use unavi_vui::{
    mesh,
    mote::{
        PipPlacement,
        Role,
    },
    view::{
        SlotView,
        Style,
    },
};
use wired_prelude::prelude::*;

use crate::wired::{
    input::{
        api::register_input_listener,
        types::{
            InputAction,
            InputListener,
        },
    },
    physics::api::get_linear_velocity,
    scene::types::{
        AlphaMode,
        Collider,
        Document,
        Material,
        Prim,
        RigidBody,
        RigidBodyKind,
        Xform,
    },
};

/// Contents sit within the body; depth marks ride outside it. That placement
/// is the whole of the down-versus-up distinction, so the two radii must stay
/// clearly apart.
const INSIDE_ORBIT: f32 = 0.52;
const AROUND_ORBIT: f32 = 1.35;
const PIP_RADIUS: f32 = 0.13;
const MARK_RADIUS: f32 = 0.09;
const OVERFLOW_RADIUS: f32 = 0.11;
const SPHERE_RINGS: usize = 10;
const SPHERE_SEGMENTS: usize = 16;

/// What a slot's pip meshes were last built for. Rebuilding costs blob
/// uploads, so nothing is re-uploaded while this is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PipShape {
    count:     usize,
    branches:  usize,
    overflow:  bool,
    placement: PipPlacement,
}

struct SlotPrims {
    root:     Prim,
    body:     Prim,
    /// Grabs on this mote, reported by the host's own hit-test against the
    /// body's collider.
    input:    InputListener,
    hit:      Cell<f32>,
    /// Container children, drawn see-through — the same rule their own motes
    /// follow one level down.
    nested:   Prim,
    /// Leaf children, or depth marks, drawn solid.
    plain:    Prim,
    overflow: Prim,
    style:    Cell<Option<Style>>,
    shape:    Cell<Option<PipShape>>,
}

pub struct Bodies {
    root:  Prim,
    slots: Vec<SlotPrims>,
}

impl Bodies {
    pub fn new(doc: &Document, capacity: usize) -> anyhow::Result<Self> {
        let root = doc.create_prim()?;
        root.set_xform(Some(placed(Vec3::ZERO, 1.0)))?;

        let unit = mesh::sphere(1.0, SPHERE_RINGS, SPHERE_SEGMENTS);
        let mut slots = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            let slot_root = doc.create_prim()?;
            slot_root.set_xform(Some(hidden()))?;
            root.add_child(&slot_root)?;

            let body = doc.create_prim()?;
            apply_mesh(&body, &unit)?;
            body.set_xform(Some(placed(Vec3::ZERO, 1.0)))?;
            slot_root.add_child(&body)?;

            let nested = doc.create_prim()?;
            nested.set_xform(Some(hidden()))?;
            slot_root.add_child(&nested)?;

            let plain = doc.create_prim()?;
            plain.set_xform(Some(hidden()))?;
            slot_root.add_child(&plain)?;

            let overflow = doc.create_prim()?;
            apply_mesh(&overflow, &mesh::overflow_marker(OVERFLOW_RADIUS))?;
            overflow.set_xform(Some(hidden()))?;
            slot_root.add_child(&overflow)?;

            // Collider and listener ride the slot root, which stays at unit
            // scale — the body's scale animates with attention and would
            // scale a collider with it.
            let input = register_input_listener(&slot_root)?;

            slots.push(SlotPrims {
                root: slot_root,
                input,
                hit: Cell::new(0.0),
                body,
                nested,
                plain,
                overflow,
                style: Cell::new(None),
                shape: Cell::new(None),
            });
        }

        Ok(Self { root, slots })
    }

    pub fn place(&self, transform: &Transform) -> anyhow::Result<()> {
        self.root.set_xform(Some(Xform {
            translation: transform.translation,
            rotation:    transform.rotation,
            scale:       transform.scale,
        }))?;
        Ok(())
    }

    /// Grabs the host reported against a mote's own collider, as
    /// `(slot, pressed)`.
    pub fn poll_grabs(&self) -> Vec<(usize, bool)> {
        let mut grabs = Vec::new();
        for (index, slot) in self.slots.iter().enumerate() {
            while let Some(event) = slot.input.poll() {
                match event.action {
                    InputAction::GrabDown => grabs.push((index, true)),
                    InputAction::GrabUp => grabs.push((index, false)),
                    _ => {}
                }
            }
        }
        grabs
    }

    /// Hands a mote to the engine's grab, which is watching for a body to
    /// appear for a moment after the squeeze.
    pub fn make_dynamic(&self, slot: usize) -> anyhow::Result<()> {
        let Some(prims) = self.slots.get(slot) else {
            return Ok(());
        };
        prims.root.set_rigid_body(Some(RigidBody {
            kind:            RigidBodyKind::Dynamic,
            angular_damping: None,
            friction:        None,
            linear_damping:  None,
            mass:            None,
            restitution:     None,
        }))?;
        Ok(())
    }

    pub fn clear_dynamic(&self, slot: usize) -> anyhow::Result<()> {
        if let Some(prims) = self.slots.get(slot) {
            prims.root.set_rigid_body(None)?;
            // Re-issue the collider next frame.
            prims.hit.set(0.0);
        }
        Ok(())
    }

    #[must_use]
    pub fn pose(&self, slot: usize) -> Option<Transform> {
        self.slots.get(slot).map(|prims| prims.root.global_xform())
    }

    #[must_use]
    pub fn velocity(&self, slot: usize) -> Vec3 {
        self.slots
            .get(slot)
            .and_then(|prims| get_linear_velocity(&prims.root).ok())
            .unwrap_or(Vec3::ZERO)
    }

    /// `held` is the slot the engine is carrying, whose transform belongs to
    /// the solver rather than to us.
    pub fn apply(&self, views: &[SlotView], held: Option<usize>) -> anyhow::Result<()> {
        for (index, (slot, view)) in self.slots.iter().zip(views).enumerate() {
            if held != Some(index) {
                slot.root.set_xform(Some(placed(view.position, 1.0)))?;
            }
            slot.body.set_xform(Some(placed(Vec3::ZERO, view.radius)))?;

            if (slot.hit.get() - view.hit_radius).abs() > f32::EPSILON {
                slot.hit.set(view.hit_radius);
                slot.root
                    .set_collider(Some(Collider::Sphere(view.hit_radius)))?;
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

        for slot in self.slots.iter().skip(views.len()) {
            slot.root.set_xform(Some(hidden()))?;
        }
        Ok(())
    }

    /// Drawn unconditionally, not on attention: how much a container holds is
    /// structural.
    fn apply_pips(slot: &SlotPrims, view: &SlotView) -> anyhow::Result<()> {
        let branches = view.pips.branches();
        let shape = PipShape {
            count: view.pips.count,
            branches,
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
            apply_run(&slot.nested, 0, branches, total, ring, radius)?;
            apply_run(&slot.plain, branches, total - branches, total, ring, radius)?;
        }

        let visible = placed(Vec3::ZERO, view.radius);
        for (prim, shown) in [
            (&slot.nested, branches > 0),
            (&slot.plain, view.pips.count > branches),
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
    let opaque = matches!(role, Role::Leaf | Role::Cast | Role::Parent { .. });
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
