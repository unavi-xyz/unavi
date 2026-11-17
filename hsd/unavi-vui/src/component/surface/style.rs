//! The style a mote is drawn in at a given heat.

use wired_guest::math::Color;

use super::Surface;
use crate::{
    part::{
        spec::Role,
        view::Style,
    },
    tokens::{
        palette::{
            self,
        },
        state::{
            self,
            Attention,
            Condition,
        },
    },
};

impl Surface {
    /// A role's style sampled at `heat`, by blending the two attention states
    /// it falls between.
    ///
    /// Blending the finished styles rather than each palette value keeps every
    /// rule in [`Surface::style`] — a parent mote receding, a container's
    /// glass, an item's warmth — exactly what it was at each state, and none
    /// of them has to know heat exists.
    /// `crowd` is how much of its brightness the mote keeps while something
    /// else holds attention; exactly 1 leaves the sampled style untouched.
    pub(super) fn style_at(
        &self,
        role: Role,
        heat: f32,
        condition: Condition,
        tint: Option<Color>,
        crowd: f32,
    ) -> Style {
        let (low, high, t) = state::bracket(heat);
        // Landing on an endpoint exactly, rather than blending to it: a style
        // is only rewritten when it differs, and `a + (b - a) * 1.0` is not
        // `b` in floating point — so a settled mote would redraw forever.
        let sampled = if t <= 0.0 {
            self.style(role, low, condition, tint)
        } else if t >= 1.0 {
            self.style(role, high, condition, tint)
        } else {
            let (from, to) = (
                self.style(role, low, condition, tint),
                self.style(role, high, condition, tint),
            );
            Style {
                color:    palette::blend(from.color, to.color, t),
                alpha:    (to.alpha - from.alpha).mul_add(t, from.alpha),
                emissive: (to.emissive - from.emissive).mul_add(t, from.emissive),
            }
        };
        Style {
            emissive: sampled.emissive * crowd,
            ..sampled
        }
    }

    pub(super) const fn style(
        &self,
        role: Role,
        attention: Attention,
        condition: Condition,
        tint: Option<Color>,
    ) -> Style {
        // A mote's own hue is what it *is*, so it beats every resting rule —
        // the source's warmth and the parent's dim both belong to the neutral
        // family. What is in hand keeps the hue too, lit up by the grip
        // rather than recoloured: the accent's grey reads as a mote going
        // dead when it is selected, where a glow keeps it alive.
        let color = match (role, attention, tint) {
            (_, Attention::Engaged, Some(tint)) => self.palette.tinted(tint, attention),
            (_, Attention::Engaged, _) => self.palette.tint(attention),
            // An off toggle keeps its hue but sits back from it, so the tools
            // of a ring stay told apart while the fill says which are on.
            (Role::Toggle, _, Some(tint)) if !condition.active => {
                self.palette.tinted(self.palette.muted(tint), attention)
            }
            (Role::Toggle, ..) if !condition.active => self.palette.dim,
            (_, _, Some(tint)) => self.palette.tinted(tint, attention),
            (Role::Parent { .. } | Role::Page { .. }, ..) if !attention.is_active() => {
                self.palette.dim
            }
            (Role::Item { .. }, ..) => self.palette.item(attention, role.is_source()),
            (_, _, None) => self.palette.tint(attention),
        };
        Style {
            color,
            alpha: match role {
                // A toggle's fill *is* the signal: on is the mote wearing its
                // hue, off is the same glass as everything else.
                Role::Toggle if condition.active => self.palette.toggle_active_alpha,
                Role::Toggle
                | Role::Group { .. }
                | Role::Action
                | Role::Item { .. }
                | Role::Cast
                | Role::Parent { .. }
                | Role::Page { .. } => self.palette.glass(attention),
            },
            emissive: self.palette.emissive_lit(attention, condition),
        }
    }
}
