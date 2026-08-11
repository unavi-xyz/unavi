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
    pub base:    Color,
    /// The one saturated hue, spent only on what has attention.
    pub accent:  Color,
    /// Receded: the way back, and anything inactive.
    pub dim:     Color,
    /// Backing surfaces — racks, tables, cast circles.
    pub surface: Color,
    /// Per-[`MoteKind`] hue, indexed by [`MoteKind::index`]. Kept close to
    /// `base` on purpose: identity is carried by silhouette, and colour is
    /// reserved for state.
    pub kinds:   [Color; MoteKind::COUNT],

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
    /// One cool near-white family at eight lightnesses. Kinds vary by shade,
    /// never by hue — identity is carried by silhouette.
    pub const DEFAULT: Self = Self {
        base:    shade(0.92),
        accent:  rgb(0.60, 0.84, 1.00),
        dim:     shade(0.42),
        surface: rgb(0.12, 0.13, 0.16),
        kinds:   [
            shade(0.97),
            shade(0.90),
            shade(0.83),
            shade(0.77),
            shade(0.71),
            shade(0.65),
            shade(0.59),
            shade(0.53),
        ],

        glass_alpha:          0.16,
        glass_alpha_attended: 0.34,
        solid_alpha:          0.94,

        emissive_base:     0.08,
        emissive_near:     0.18,
        emissive_attended: 0.42,
        emissive_engaged:  0.70,
    };

    #[must_use]
    pub const fn kind(&self, kind: MoteKind) -> Color {
        self.kinds[kind.index()]
    }

    /// Attention lifts toward white rather than repainting, so the accent
    /// stays free for the rarer state of being in hand.
    #[must_use]
    pub const fn tint(&self, kind: MoteKind, attention: Attention) -> Color {
        match attention {
            Attention::Engaged => self.accent,
            Attention::Attended => lift(self.kind(kind), 0.55),
            Attention::Near => lift(self.kind(kind), 0.18),
            Attention::Idle => self.kind(kind),
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

/// One step of the neutral family: a cool near-white at `lightness`.
#[must_use]
pub const fn shade(lightness: f32) -> Color {
    Color {
        r: lightness * 0.96,
        g: lightness * 0.98,
        b: lightness,
        a: 1.0,
    }
}

/// Moves a colour `amount` of the way to white, keeping its hue.
#[must_use]
pub const fn lift(color: Color, amount: f32) -> Color {
    Color {
        r: color.r + (1.0 - color.r) * amount,
        g: color.g + (1.0 - color.g) * amount,
        b: color.b + (1.0 - color.b) * amount,
        a: color.a,
    }
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

    fn lightness(color: Color) -> f32 {
        (color.r + color.g + color.b) / 3.0
    }

    fn spread(color: Color) -> f32 {
        color.r.max(color.g).max(color.b) - color.r.min(color.g).min(color.b)
    }

    #[test]
    fn attention_brightens_rather_than_repainting() {
        let palette = Palette::DEFAULT;
        let idle = palette.tint(MoteKind::Tool, Attention::Idle);
        let near = palette.tint(MoteKind::Tool, Attention::Near);
        let attended = palette.tint(MoteKind::Tool, Attention::Attended);

        assert!(lightness(idle) < lightness(near));
        assert!(lightness(near) < lightness(attended));
        for lifted in [near, attended] {
            assert!(
                spread(lifted) <= spread(idle) + 1.0e-5,
                "a lift must not introduce a hue"
            );
        }
    }

    #[test]
    fn the_accent_is_reserved_for_what_is_in_hand() {
        let palette = Palette::DEFAULT;
        assert_eq!(
            palette.tint(MoteKind::Tool, Attention::Engaged),
            palette.accent
        );
        for quiet in [Attention::Idle, Attention::Near, Attention::Attended] {
            assert_ne!(
                palette.tint(MoteKind::Tool, quiet),
                palette.accent,
                "hover is not rare enough to spend the accent on"
            );
        }
    }

    #[test]
    fn kinds_differ_by_shade_and_never_by_hue() {
        let palette = Palette::DEFAULT;
        let reference = spread(palette.kind(MoteKind::Command));
        for kind in ALL {
            assert!(
                (spread(palette.kind(kind)) - reference).abs() < 0.02,
                "{kind:?} is a different hue, not a different shade"
            );
        }
        let lightnesses = ALL.map(|kind| lightness(palette.kind(kind)));
        for pair in lightnesses.windows(2) {
            assert!(pair[0] > pair[1], "shades must stay distinguishable");
        }
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
            custom.tint(MoteKind::Item, Attention::Engaged),
            rgb(0.0, 1.0, 0.6)
        );
    }
}
