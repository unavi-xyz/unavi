use wired_math::types::{
    Transform,
    Vec2,
    Vec3,
};
use wired_scene::types::Color;

use crate::{
    attention::Attention,
    mote::{
        Pips,
        Role,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub color:    Color,
    pub alpha:    f32,
    pub emissive: f32,
}

/// Everything a renderer needs for one slot, in concrete values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotView {
    /// Surface-local, lean already applied.
    pub position:     Vec3,
    pub radius:       f32,
    pub style:        Style,
    pub role:         Role,
    pub attention:    Attention,
    /// How far this mote has come toward [`SlotView::attention`]. The state
    /// says which mote is attended; this says how far along saying so it is,
    /// and it is what the shell's rim widens on.
    pub heat:         f32,
    pub pips:         Pips,
    /// The body has left its slot and is following the hand. A merely pressed
    /// mote is not this — see [`Attention::Engaged`].
    pub seized:       bool,
    /// Where this mote's name goes, slot-local. The text itself comes from
    /// the spec, not from this view.
    pub label_offset: Vec3,
    pub label_size:   f32,
    /// How far this mote has arrived, scaling everything drawn for it. The
    /// form's own drawing-in is already in [`SlotView::position`].
    pub bloom:        f32,
}

/// Which window of an oversized level is currently drawn.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PageView {
    pub index:   usize,
    pub count:   usize,
    /// Specs before the window, which the leading pinned slots do not cover.
    pub skipped: usize,
    pub total:   usize,
}

impl PageView {
    #[must_use]
    pub const fn is_paged(&self) -> bool {
        self.count > 1
    }

    #[must_use]
    pub const fn has_previous(&self) -> bool {
        self.index > 0
    }

    #[must_use]
    pub const fn has_next(&self) -> bool {
        self.index + 1 < self.count
    }
}

/// Where the pointer is, in the surface's own coordinates and in the world.
#[derive(Debug, Clone, Copy)]
pub struct Aim {
    pub local: Vec2,
    pub world: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct Frame {
    pub eye:    Vec3,
    pub anchor: Transform,
    /// Resolves which slot is targeted. A held mote follows [`Frame::hand`]
    /// instead.
    pub aim:    Option<Aim>,
    /// Free world-space grab point; a held mote follows this rather than
    /// [`Frame::aim`].
    pub hand:   Option<Vec3>,
    pub delta:  f32,
}
