use std::cell::Cell;

use wired_prelude::prelude::*;

use crate::{
    palette,
    unavi::shapes::api::Cylinder,
    wired::scene::{
        api::self_document,
        types::{
            GraphValue,
            Prim,
            Xform,
        },
    },
};

/// Beam radius. Thin enough to read as a line; the glow comes from the
/// graph's additive core, not from the tube being fat.
const WIDTH: f32 = 0.004;
/// Rings along the beam. The rope drag is vertex displacement, so a
/// two-vertex cuboid could not bend at all.
const SEGMENTS: u32 = 24;
const RESOLUTION: u32 = 6;

/// Fraction of the frame's midpoint movement that becomes rope drag. The
/// beam is straight at rest and only bows while the muzzle is moving.
const DRAG: f32 = 0.18;
/// Per-frame decay back to straight once movement stops.
const RECOVER: f32 = 0.5;
/// Metres of bow the drag may reach, so a fast flick bends the rope rather
/// than throwing the midpoint away. Most of the visible trailing now comes
/// from the prop lagging (`hold::FOLLOW`), not from bowing the rope.
const MAX_DRAG: f32 = 0.16;

/// The authored prim carrying the compiled beam graph. It has no mesh and
/// renders nothing; it exists so a runtime-created prim can bind to a graph
/// the package already carries, since a script cannot author one.
const TEMPLATE_PRIM_NAME: &str = "beam_template";
/// `beam.hss` public input 0.
const TINT_INPUT: u16 = 0;
/// `beam.hss` public input 4: rope drag, a world-space offset applied at the
/// beam's midpoint.
const DRAG_INPUT: u16 = 4;

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

fn clamp_length(v: Vec3, max: f32) -> Vec3 {
    let len = v.length();
    if len > max { v * (max / len) } else { v }
}

/// A segmented cylinder stretched between the muzzle and the grab point,
/// bowed by `beam.hss` while it is being dragged around.
///
/// Only the endpoints and the drag offset move per frame; the glow and the
/// travelling energy are fragment work driven by the view's own clock.
pub struct Laser {
    prim:   Prim,
    color:  Cell<Option<Color>>,
    /// Where the midpoint would be if the rope were rigid, last frame.
    anchor: Cell<Option<Vec3>>,
    /// Current bow, in world metres. Zero means a straight beam.
    drag:   Cell<Vec3>,
}

impl Laser {
    #[must_use]
    pub fn new() -> Self {
        let doc = self_document().expect("self_document");

        let cylinder = Cylinder::new(1.0, 1.0);
        cylinder.set_resolution(RESOLUTION);
        cylinder.set_segments(SEGMENTS);
        cylinder.set_doc(doc.clone());

        let prim = cylinder.mesh();
        prim.set_xform(Some(hidden())).ok();

        match doc
            .prims()
            .into_iter()
            .find(|p| p.name().is_some_and(|n| n == TEMPLATE_PRIM_NAME))
        {
            Some(template) => {
                prim.set_relationship("material:binding", Some(&template.id()))
                    .ok();
            }
            None => eprintln!("physgun: HSD missing {TEMPLATE_PRIM_NAME} prim; beam unshaded"),
        }

        Self {
            prim,
            color: Cell::new(None),
            anchor: Cell::new(None),
            drag: Cell::new(Vec3::ZERO),
        }
    }

    pub fn show(&self, from: Vec3, to: Vec3, color: Color) {
        if self.color.get() != Some(color) {
            self.color.set(Some(color));
            self.push_overrides();
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
                scale:       Vec3::new(WIDTH, len, WIDTH),
            }))
            .ok();

        self.update_drag((from + to) * 0.5);
    }

    /// Rope drag: the midpoint lags behind where a rigid beam would put it,
    /// then eases back to straight. Sampled from actual movement rather than
    /// animated, so a stationary beam is perfectly straight.
    fn update_drag(&self, midpoint: Vec3) {
        let previous = self.anchor.replace(Some(midpoint));
        let moved = previous.map_or(Vec3::ZERO, |p| p - midpoint);

        let drag = clamp_length(self.drag.get() + moved * DRAG, MAX_DRAG) * (1.0 - RECOVER);
        let drag = if drag.length() < 1.0e-4 {
            Vec3::ZERO
        } else {
            drag
        };

        if self.drag.replace(drag) != drag {
            self.push_overrides();
        }
    }

    /// Writes every override at once: the host call replaces the whole map,
    /// so sending one input alone would clear the others.
    ///
    /// The graph itself is shared with every other physgun beam — only these
    /// values differ, so this re-uploads two `vec4`s rather than recompiling
    /// anything.
    fn push_overrides(&self) {
        let tint = palette::beam_tint(self.color.get().unwrap_or(palette::DEFAULT));
        let drag = self.drag.get();
        self.prim
            .set_graph_overrides(&[
                (TINT_INPUT, GraphValue::Color(tint)),
                (DRAG_INPUT, GraphValue::Vec3(drag)),
            ])
            .ok();
    }

    pub fn hide(&self) {
        self.prim.set_xform(Some(hidden())).ok();
        self.anchor.set(None);
        self.drag.set(Vec3::ZERO);
    }
}
