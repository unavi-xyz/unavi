use std::{
    cell::{
        Cell,
        RefCell,
    },
    f32::consts::PI,
};

use unavi_gauntlet_menu::{
    Icon,
    Slot,
};
use wired_prelude::prelude::*;

use crate::{
    geometry::{
        self,
        ICON_R,
        MeshData,
        RING_RADIUS,
        SECTOR_INNER_R,
    },
    palette,
    wired::scene::{
        api::self_document,
        types::{
            Material,
            Prim,
            Xform,
        },
    },
};

pub const RAISE_DIST: f32 = 0.022;
pub const RAISE_SPEED: f32 = 12.0;
const MAX_SLOTS: usize = 12;

const IDENTITY: Quat = Quat {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 1.0,
};

const fn scaled(scale: f32) -> Xform {
    Xform {
        translation: Vec3::ZERO,
        rotation:    IDENTITY,
        scale:       Vec3::splat(scale),
    }
}

fn placed(translation: Vec3) -> Xform {
    Xform {
        translation,
        rotation: IDENTITY,
        scale: Vec3::ONE,
    }
}

/// A persistent sector slot. Prims are created once and reconfigured on rebuild,
/// so nothing is ever spawned mid-session (avoids the one-frame origin flash of
/// freshly created prims).
struct SlotPrims {
    root:      Prim,
    bg:        Prim,
    outline:   Prim,
    glyph:     Prim,
    base_mat:  RefCell<Material>,
    hover_mat: RefCell<Material>,
    hovered:   Cell<bool>,
    raise_t:   Cell<f32>,
}

impl SlotPrims {
    fn set_hovered(&self, hovered: bool) {
        if self.hovered.get() != hovered {
            self.hovered.set(hovered);
            let mat = if hovered {
                self.hover_mat.borrow()
            } else {
                self.base_mat.borrow()
            };
            self.bg.set_material(Some(&mat));
        }
    }

    fn set_raise(&self, raise: f32) {
        self.raise_t.set(raise);
        self.root.set_xform(Some(Xform {
            translation: Vec3::new(0.0, 0.0, raise * RAISE_DIST),
            rotation:    IDENTITY,
            scale:       Vec3::ONE,
        }));
    }

    fn hide(&self) {
        self.root.set_xform(Some(scaled(0.0)));
    }

    fn configure(&self, i: usize, n: usize, slot: &Slot) {
        let color = icon_color(slot.icon, slot.active);
        *self.base_mat.borrow_mut() =
            palette::glass(color, palette::GLASS_ALPHA, palette::EMISSIVE_BASE);
        *self.hover_mat.borrow_mut() =
            palette::glass(color, palette::GLASS_ALPHA_HOVER, palette::EMISSIVE_HOVER);

        geometry::apply_mesh(&self.bg, &geometry::sector_mesh(i, n));
        let base = self.base_mat.borrow();
        self.bg.set_material(Some(&base));
        drop(base);

        if slot.active {
            geometry::apply_mesh(&self.outline, &geometry::outline_mesh(i, n));
            self.outline
                .set_material(Some(&palette::solid(palette::ACCENT, 0.7)));
            self.outline.set_xform(Some(scaled(1.0)));
        } else {
            self.outline.set_xform(Some(scaled(0.0)));
        }

        geometry::apply_mesh(&self.glyph, &icon_mesh(slot.icon));
        self.glyph.set_material(Some(&palette::solid(color, 0.6)));
        let angle = i as f32 * 2.0 * PI / n as f32;
        self.glyph.set_xform(Some(placed(Vec3::new(
            ICON_R * angle.cos(),
            ICON_R * angle.sin(),
            0.006,
        ))));

        self.hovered.set(false);
        self.set_raise(0.0);
    }
}

pub struct Wheel {
    pub root: Prim,
    slots:    Vec<SlotPrims>,
    count:    Cell<usize>,
}

impl Wheel {
    #[must_use]
    pub fn new() -> Self {
        let doc = self_document().expect("self_document");
        let root = doc.create_prim();
        root.set_xform(Some(scaled(0.0)));

        let slots = (0..MAX_SLOTS)
            .map(|_| {
                let slot_root = doc.create_prim();
                slot_root.set_xform(Some(scaled(0.0)));
                let bg = doc.create_prim();
                bg.set_xform(Some(scaled(1.0)));
                let outline = doc.create_prim();
                outline.set_xform(Some(scaled(0.0)));
                let glyph = doc.create_prim();
                glyph.set_xform(Some(scaled(1.0)));
                slot_root.add_child(&bg);
                slot_root.add_child(&outline);
                slot_root.add_child(&glyph);
                root.add_child(&slot_root);
                SlotPrims {
                    root: slot_root,
                    bg,
                    outline,
                    glyph,
                    base_mat: RefCell::new(palette::glass(
                        palette::NEUTRAL,
                        palette::GLASS_ALPHA,
                        palette::EMISSIVE_BASE,
                    )),
                    hover_mat: RefCell::new(palette::glass(
                        palette::NEUTRAL,
                        palette::GLASS_ALPHA_HOVER,
                        palette::EMISSIVE_HOVER,
                    )),
                    hovered: Cell::new(false),
                    raise_t: Cell::new(0.0),
                }
            })
            .collect();

        Self {
            root,
            slots,
            count: Cell::new(0),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.count.get()
    }

    pub fn rebuild(&self, slots: &[Slot]) {
        let count = slots.len().min(MAX_SLOTS);
        self.count.set(count);
        for (i, slot) in slots.iter().take(count).enumerate() {
            self.slots[i].configure(i, count, slot);
        }
        for slot in self.slots.iter().skip(count) {
            slot.hide();
        }
    }

    pub fn animate(&self, delta: f32, hovered: Option<usize>) {
        let step = delta * RAISE_SPEED;
        for (i, slot) in self.slots.iter().take(self.count.get()).enumerate() {
            let is_hovered = Some(i) == hovered;
            slot.set_hovered(is_hovered);
            let target = if is_hovered { 1.0 } else { 0.0 };
            let cur = slot.raise_t.get();
            let next = if target > cur {
                (cur + step).min(target)
            } else {
                (cur - step).max(target)
            };
            if (next - cur).abs() > f32::EPSILON {
                slot.set_raise(next);
            }
        }
    }
}

impl Default for Wheel {
    fn default() -> Self {
        Self::new()
    }
}

fn icon_color(icon: Icon, active: bool) -> Color {
    match icon {
        Icon::Home | Icon::Confirm => palette::ACCENT,
        Icon::Tools => palette::SECONDARY,
        Icon::Back => palette::DIM,
        Icon::Tool if active => palette::ACCENT,
        Icon::Tool => palette::NEUTRAL,
    }
}

fn icon_mesh(icon: Icon) -> MeshData {
    match icon {
        Icon::Home => geometry::home_mesh(),
        Icon::Tools => geometry::gear_mesh(),
        Icon::Back => geometry::chevron_mesh(),
        Icon::Confirm => geometry::check_mesh(),
        Icon::Tool => geometry::diamond_mesh(),
    }
}

/// Projects the `forward` ray from `origin` onto the wheel plane and returns the
/// index of the sector the cursor falls in, if any.
#[must_use]
pub fn hovered_sector(
    origin: Vec3,
    forward: Vec3,
    plane_pos: Vec3,
    plane_rot: Quat,
    sector_count: usize,
) -> Option<usize> {
    if sector_count == 0 {
        return None;
    }

    let normal = plane_rot * Vec3::Z;
    let denom = forward.dot(normal);
    if denom.abs() < 1.0e-6 {
        return None;
    }
    let t = (plane_pos - origin).dot(normal) / denom;
    if t < 0.0 {
        return None;
    }

    let rel = origin + forward * t - plane_pos;
    let right = plane_rot * Vec3::X;
    let up = plane_rot * Vec3::Y;
    let x = rel.dot(right);
    let y = rel.dot(up);
    let dist = x.hypot(y);
    if dist < SECTOR_INNER_R || dist > RING_RADIUS + 0.05 {
        return None;
    }

    let mut angle = y.atan2(x);
    if angle < 0.0 {
        angle += 2.0 * PI;
    }
    let n = sector_count;
    let sector = (angle * n as f32 / (2.0 * PI)).round() as usize % n;
    Some(sector)
}
