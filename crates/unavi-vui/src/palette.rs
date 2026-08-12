use wired_scene::types::Color;

use crate::attention::Attention;

/// Every colour and surface value VUI draws with.
///
/// Primitives read this rather than holding colours of their own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Near-white default.
    pub base:    Color,
    /// The saturated hue, used only when engaged.
    pub accent:  Color,
    /// Receded: inactive motes and the way back.
    pub dim:     Color,
    /// Backing surfaces — racks, tables, cast circles, placards.
    pub surface: Color,

    /// A container is see-through.
    pub glass_alpha:          f32,
    pub glass_alpha_attended: f32,
    /// A leaf is solid.
    pub solid_alpha:          f32,

    pub emissive_base:     f32,
    pub emissive_near:     f32,
    pub emissive_attended: f32,
    pub emissive_engaged:  f32,
}

impl Palette {
    /// A cool near-white family; colour only signals state.
    pub const DEFAULT: Self = Self {
        base:    shade(0.92),
        accent:  rgb(0.60, 0.84, 1.00),
        dim:     shade(0.42),
        surface: rgb(0.12, 0.13, 0.16),

        glass_alpha:          0.16,
        glass_alpha_attended: 0.34,
        solid_alpha:          0.94,

        emissive_base:     0.08,
        emissive_near:     0.18,
        emissive_attended: 0.42,
        emissive_engaged:  0.70,
    };

    #[must_use]
    /// Attention lifts toward white rather than repainting.
    pub const fn tint(&self, attention: Attention) -> Color {
        match attention {
            Attention::Engaged => self.accent,
            Attention::Attended => lift(self.base, 0.55),
            Attention::Near => lift(self.base, 0.18),
            Attention::Idle => self.base,
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

    fn lightness(color: Color) -> f32 {
        (color.r + color.g + color.b) / 3.0
    }

    fn spread(color: Color) -> f32 {
        color.r.max(color.g).max(color.b) - color.r.min(color.g).min(color.b)
    }

    #[test]
    fn attention_brightens_rather_than_repainting() {
        let palette = Palette::DEFAULT;
        let idle = palette.tint(Attention::Idle);
        let near = palette.tint(Attention::Near);
        let attended = palette.tint(Attention::Attended);

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
        assert_eq!(palette.tint(Attention::Engaged), palette.accent);
        for quiet in [Attention::Idle, Attention::Near, Attention::Attended] {
            assert_ne!(
                palette.tint(quiet),
                palette.accent,
                "hover is not rare enough to spend the accent on"
            );
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
        for quiet in [palette.base, palette.dim, palette.surface] {
            assert!(
                spread(quiet) < spread(palette.accent),
                "a resting colour must not compete with the accent"
            );
        }
    }

    #[test]
    fn a_consumer_can_restyle_without_touching_primitives() {
        let custom = Palette {
            accent: rgb(0.0, 1.0, 0.6),
            ..Palette::DEFAULT
        };
        assert_eq!(custom.tint(Attention::Engaged), rgb(0.0, 1.0, 0.6));
    }
}
