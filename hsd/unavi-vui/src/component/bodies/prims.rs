//! The prims one slot owns.

use std::cell::{
    Cell,
    RefCell,
};

use smol_str::SmolStr;
use wired_guest::math::{
    Transform,
    Vec3,
};

use super::face::Face;
use crate::{
    component::tree::Mote,
    part::{
        draw,
        mesh,
        shadow::Shadow,
        spec::{
            Arrange,
            PipPlacement,
        },
        view::{
            Body,
            Style,
        },
    },
    tokens::state::Attention,
    wired::scene::properties::Property,
};

/// Last pip-mesh inputs. Rebuilding costs blob uploads, so nothing is
/// re-uploaded while this is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PipShape {
    pub(super) count:     usize,
    pub(super) groups:    usize,
    pub(super) overflow:  bool,
    pub(super) placement: PipPlacement,
    pub(super) arrange:   Arrange,
}

/// Everything the shell graph is handed. Not derivable from the style alone:
/// the film and frost are per-mote choices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Shell {
    pub(super) style:  Style,
    /// A toggle's mark is lit or receded by this, which the style alone does
    /// not say once the mote wears its own hue.
    pub(super) active: bool,
    pub(super) film:   f32,
    pub(super) frost:  f32,
    pub(super) bloom:  f32,
}

/// A mark a role wears in a slot's own prims, its mesh written the first time
/// the slot needs it and the prim hidden whenever it is not wanted.
pub(super) struct Mark {
    pub(super) prim:   (u64, u64),
    pub(super) carved: Cell<bool>,
    pub(super) pose:   Shadow<Option<Transform>>,
}

impl Mark {
    pub(super) const fn new(prim: (u64, u64)) -> Self {
        Self {
            prim,
            carved: Cell::new(false),
            pose: Shadow::new(),
        }
    }

    /// Takes back what was written, for a frame whose edits did not land.
    pub(super) fn forget(&self) {
        self.carved.set(false);
        self.pose.clear();
    }

    pub(super) fn apply<'a>(
        &self,
        mut batch: crate::Batch<'a>,
        build: fn() -> mesh::MeshData,
        wanted: Option<Transform>,
    ) -> crate::Batch<'a> {
        if wanted.is_some() && !self.carved.replace(true) {
            batch = draw::mesh(&build())
                .into_iter()
                .fold(batch, |batch, property| batch.set(self.prim, property));
        }
        match self.pose.diff(wanted) {
            Some(pose) => batch.set(
                self.prim,
                Property::Transform(pose.unwrap_or_else(draw::hidden)),
            ),
            None => batch,
        }
    }
}

/// What kind of mote a body is, marked on it: a toggle's bar and a cast's ring.
/// A group wears its pips instead.
pub(super) struct Marks {
    pub(super) bar:  Mark,
    pub(super) ring: Mark,
}

pub(super) struct SlotPrims {
    pub(super) root:        (u64, u64),
    pub(super) root_xform:  Shadow<Transform>,
    pub(super) body:        (u64, u64),
    pub(super) body_xform:  Shadow<Transform>,
    /// The shape the body's mesh was last written as. A slot is made an orb
    /// and becomes a key when a bed first puts one there.
    pub(super) body_shape:  Shadow<Body>,
    /// Container children, drawn see-through.
    pub(super) nested:      (u64, u64),
    /// Leaf children, or depth marks, drawn solid.
    pub(super) plain:       (u64, u64),
    pub(super) overflow:    (u64, u64),
    /// Whether `overflow` has had its mesh written yet.
    pub(super) marked:      Cell<bool>,
    /// The mote's name, drawn always rather than on attention.
    pub(super) label:       (u64, u64),
    pub(super) marks:       Marks,
    /// The mote's glyph, when it has one, drawn in place of its icon.
    pub(super) face:        Face,
    pub(super) label_xform: Shadow<Option<Vec3>>,
    /// The mote whose icon is parented here, so a slot reused by another mote
    /// hands the old one back rather than keeping it.
    pub(super) icon:        RefCell<Option<Mote>>,
    /// What the shell graph was last handed, so a settled mote is not written
    /// every frame.
    pub(super) shell:       Shadow<Shell>,
    pub(super) shape:       Shadow<PipShape>,
    /// Last `(label, attention)` written; a `set_text` write costs a sync
    /// whether or not the string changed.
    pub(super) written:     Shadow<(SmolStr, Attention)>,
}

impl SlotPrims {
    /// Takes back everything written to the slot's prims, so the next frame
    /// writes it all again. For a frame whose batch failed to flush.
    pub(super) fn forget(&self) {
        self.root_xform.clear();
        self.body_xform.clear();
        self.body_shape.clear();
        self.marked.set(false);
        self.label_xform.clear();
        self.shell.clear();
        self.shape.clear();
        self.written.clear();
        self.marks.bar.forget();
        self.marks.ring.forget();
        self.face.forget();
    }
}
