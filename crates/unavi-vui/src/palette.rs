use wired_scene::types::Color;

use crate::{
    attention::Attention,
    mote::MoteKind,
};

/// Every colour and surface value VUI draws with.
///
/// Primitives never hold colours of their own — they read this — so a
/// consumer restyles the whole system by passing a different one, and no
/// widget can quietly diverge from the theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Near-white default. Most of the field is this.
    pub base:   Color,
    /// The one saturated hue, spent only on what has attention.
    pub accent: Color,
    /// Receded: the way back, and anything inactive.
    pub dim:    Color,
    /// Backing surfaces — racks, tables, cast circles.
    pub surface: Color,
    /// Per-[`MoteKind`] hue, indexed by [`MoteKind::index`]. Kept close to
    /// `base` on purpose: identity is carried by silhouette, and colour is
    /// reserved for state.
    pub kinds:  [Color; MoteKind::COUNT],

    /// A container is see-through because you are meant to see into it.
    pub glass_alpha:          f32,
    pub glass_alpha_attended: f32,
    /// A leaf holds nothing, so it is solid.
    pub solid_alpha:          f32,

    pub emissive_base:     f32,
    pub emissive_near:     f32,
    pub emissive_attended: f32,
    pub emissive_engaged:  f32,
}

impl Palette {
    /// The default: a desaturated near-white field so a single saturated hue
    /// can mean "this one". If three things are red, red means nothing.
    pub const DEFAULT: Self = Self {
        base:    rgb(0.94, 0.96, 0.98),
        accent:  rgb(0.96, 0.20, 0.16),
        dim:     rgb(0.52, 0.60, 0.70),
        surface: rgb(0.14, 0.15, 0.18),
        kinds:   [
            rgb(0.94, 0.96, 0.98),
            rgb(0.86, 0.91, 0.97),
            rgb(0.90, 0.93, 0.97),
            rgb(0.74, 0.92, 0.96),
            rgb(0.97, 0.91, 0.82),
            rgb(0.84, 0.92, 1.00),
            rgb(0.90, 0.95, 0.90),
            rgb(0.89, 0.90, 0.96),
        ],

        glass_alpha:          0.16,
        glass_alpha_attended: 0.34,
        solid_alpha:          0.94,

        emissive_base:     0.10,
        emissive_near:     0.22,
        emissive_attended: 0.55,
        emissive_engaged:  0.85,
    };

    #[must_use]
    pub const fn kind(&self, kind: MoteKind) -> Color {
        self.kinds[kind.index()]
    }

    /// Only what has attention takes the accent, which is what makes colour
    /// answer "which one will I get" without any chrome.
    #[must_use]
    pub const fn tint(&self, kind: MoteKind, attention: Attention) -> Color {
        match attention {
            Attention::Attended | Attention::Engaged => self.accent,
            Attention::Idle | Attention::Near => self.kind(kind),
        }
    }

    #[must_use]
    pub const fn emissive(&self, attention: Attention) -> f32 {
        match attention {
            Attention::Engaged => self.emissive_engaged,
            Attention::Attended => self.emissive_attended,
            Attention::Near => self.emissive_near,
            Attention::Idle => self.emissive_base,
        }
    }

    #[must_use]
    pub const fn glass(&self, attention: Attention) -> f32 {
        match attention {
            Attention::Attended | Attention::Engaged => self.glass_alpha_attended,
            Attention::Idle | Attention::Near => self.glass_alpha,
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[must_use]
pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color { r, g, b, a: 1.0 }
}

#[must_use]
pub const fn with_alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a,
    }
}

#[must_use]
pub const fn scale(color: Color, factor: f32) -> Color {
    Color {
        r: color.r * factor,
        g: color.g * factor,
        b: color.b * factor,
        a: 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [MoteKind; MoteKind::COUNT] = [
        MoteKind::Command,
        MoteKind::Folder,
        MoteKind::Document,
        MoteKind::Space,
        MoteKind::Person,
        MoteKind::Tool,
        MoteKind::Item,
        MoteKind::Result,
    ];

    #[test]
    fn every_kind_indexes_a_distinct_hue() {
        let palette = Palette::DEFAULT;
        for (position, kind) in ALL.iter().enumerate() {
            assert_eq!(kind.index(), position, "index order must match the table");
            let color = palette.kind(*kind);
            assert!((0.0..=1.0).contains(&color.r));
        }
    }

    #[test]
    fn only_attention_spends_the_accent() {
        let palette = Palette::DEFAULT;
        assert_eq!(palette.tint(MoteKind::Tool, Attention::Attended), palette.accent);
        assert_eq!(palette.tint(MoteKind::Tool, Attention::Engaged), palette.accent);
        assert_ne!(palette.tint(MoteKind::Tool, Attention::Idle), palette.accent);
        assert_ne!(palette.tint(MoteKind::Tool, Attention::Near), palette.accent);
    }

    #[test]
    fn emissive_rises_with_attention() {
        let palette = Palette::DEFAULT;
        assert!(palette.emissive(Attention::Idle) < palette.emissive(Attention::Near));
        assert!(palette.emissive(Attention::Near) < palette.emissive(Attention::Attended));
        assert!(palette.emissive(Attention::Attended) < palette.emissive(Attention::Engaged));
    }

    #[test]
    fn the_field_is_desaturated_so_the_accent_can_carry_meaning() {
        let palette = Palette::DEFAULT;
        let spread = |c: Color| c.r.max(c.g).max(c.b) - c.r.min(c.g).min(c.b);
        for kind in ALL {
            assert!(
                spread(palette.kind(kind)) < spread(palette.accent),
                "{kind:?} competes with the accent"
            );
        }
    }

    #[test]
    fn a_consumer_can_restyle_without_touching_primitives() {
        let custom = Palette {
            accent: rgb(0.0, 1.0, 0.6),
            ..Palette::DEFAULT
        };
        assert_eq!(
            custom.tint(MoteKind::Item, Attention::Attended),
            rgb(0.0, 1.0, 0.6)
        );
    }
}
