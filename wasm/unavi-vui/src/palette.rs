use wired_scene::types::Color;

use crate::attention::Attention;

/// How much iridescent film a mote wears unless it says otherwise. Subtle
/// enough that the identity hue still reads; a consumer raises it for the
/// full bubble or drops it for flat glass.
pub const FILM: f32 = 0.2;

/// Every colour and surface value VUI draws with.
///
/// Primitives read this rather than holding colours of their own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Near-white default.
    pub base:         Color,
    /// The saturated hue, used only when engaged.
    pub accent:       Color,
    /// Receded: inactive motes and the way back.
    pub dim:          Color,
    /// Backing surfaces — grids, tables, cast circles, placards.
    pub surface:      Color,
    /// Warmth mixed into an item that is a source of its thing, so a mote you
    /// can take another of reads apart from the one of it.
    pub source:       Color,
    /// How much of [`Palette::source`] a source carries.
    pub source_shift: f32,

    /// How strongly a container's glass wears its hue.
    pub glass_alpha:          f32,
    pub glass_alpha_attended: f32,
    /// How strongly a leaf's glass wears its hue.
    pub solid_alpha:          f32,
    /// A toggle that is off reads as a clear outline: the glass barely wears
    /// its hue, so only the rim carries the silhouette.
    pub toggle_idle_alpha:    f32,
    /// A toggle that is on fills with its hue.
    pub toggle_active_alpha:  f32,
    pub emissive_base:        f32,
    pub emissive_near:        f32,
    pub emissive_attended:    f32,
    pub emissive_engaged:     f32,
    /// The floor a mote that is on burns at. Between attended and engaged:
    /// unmistakable at rest, still outshone by what is in hand.
    pub emissive_active:      f32,
}

impl Palette {
    /// A cool near-white family; colour only signals state.
    pub const DEFAULT: Self = Self {
        base:         shade(0.92),
        accent:       rgb(0.60, 0.84, 1.00),
        dim:          shade(0.42),
        surface:      rgb(0.12, 0.13, 0.16),
        source:       rgb(1.00, 0.86, 0.62),
        source_shift: 0.30,

        glass_alpha:          0.25,
        glass_alpha_attended: 0.35,
        solid_alpha:          0.70,
        toggle_idle_alpha:    0.03,
        toggle_active_alpha:  0.70,

        emissive_base:     0.08,
        emissive_near:     0.18,
        emissive_attended: 0.42,
        emissive_engaged:  0.70,
        emissive_active:   0.55,
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

    /// The tint an item wears. A source carries a little of
    /// [`Palette::source`]; the one of a thing wears the neutral family.
    ///
    /// Grasping overrides it: the accent means what is in hand and nothing
    /// else, and warming it would say two things with one colour.
    #[must_use]
    pub const fn item(&self, attention: Attention, source: bool) -> Color {
        let tint = self.tint(attention);
        if source && !matches!(attention, Attention::Engaged) {
            blend(tint, self.source, self.source_shift)
        } else {
            tint
        }
    }

    /// A mote's own identity hue, kept as it is when attention lights it up —
    /// brighter along its own hue rather than washed toward white, which is
    /// what makes a selected mote read as *glowing* rather than grey. What is
    /// in hand still wears the accent, which says what is happening and
    /// nothing else.
    #[must_use]
    pub const fn tinted(&self, tint: Color, attention: Attention) -> Color {
        match attention {
            Attention::Engaged => self.accent,
            Attention::Attended => glow(tint, 0.5),
            Attention::Near => glow(tint, 0.18),
            Attention::Idle => tint,
        }
    }

    /// How brightly a mote burns, raised to a floor while it is on.
    ///
    /// Brightness rather than a hue: the accent means what is in hand and
    /// nothing else, so spending it here would say two things with one colour.
    #[must_use]
    pub const fn emissive_lit(&self, attention: Attention, active: bool) -> f32 {
        let resting = self.emissive(attention);
        if active && resting < self.emissive_active {
            self.emissive_active
        } else {
            resting
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

/// Moves `color` `amount` of the way toward `toward`, keeping its alpha.
#[must_use]
pub const fn blend(color: Color, toward: Color, amount: f32) -> Color {
    Color {
        r: color.r + (toward.r - color.r) * amount,
        g: color.g + (toward.g - color.g) * amount,
        b: color.b + (toward.b - color.b) * amount,
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

/// Brightens a colour along its own hue rather than toward white, so a mote
/// lights up without washing grey. When a channel would clip, the whole
/// colour is renormalized so the hue is kept exactly.
#[must_use]
pub const fn glow(color: Color, amount: f32) -> Color {
    let k = 1.0 + amount;
    let r = color.r * k;
    let g = color.g * k;
    let b = color.b * k;
    let top = r.max(g).max(b);
    if top > 1.0 {
        Color {
            r: r / top,
            g: g / top,
            b: b / top,
            a: 1.0,
        }
    } else {
        Color { r, g, b, a: 1.0 }
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

    /// Whether `a` and `b` are the same hue: a boost keeps the channel ratios
    /// exactly, where a wash toward white would pull them toward each other.
    fn same_hue(a: Color, b: Color) -> bool {
        let k = [b.r / a.r, b.g / a.g, b.b / a.b];
        k[0].is_finite() && k.iter().all(|&ratio| (ratio - k[0]).abs() < 1.0e-4)
    }

    #[test]
    fn a_consumer_s_hue_lifts_with_attention_without_becoming_the_accent() {
        let palette = Palette::DEFAULT;
        let hue = rgb(0.8, 0.2, 0.3);
        assert_eq!(palette.tinted(hue, Attention::Idle), hue);

        let near = palette.tinted(hue, Attention::Near);
        let attended = palette.tinted(hue, Attention::Attended);
        assert!(lightness(near) > lightness(hue));
        assert!(lightness(attended) > lightness(near));
        for lifted in [near, attended] {
            assert!(
                same_hue(hue, lifted),
                "attention must keep the hue, not wash it grey"
            );
        }

        assert_eq!(
            palette.tinted(hue, Attention::Engaged),
            palette.accent,
            "what is in hand says it is in hand, not what it stocks"
        );
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
    fn a_source_shifts_off_the_neutral_family_without_leaving_it() {
        let palette = Palette::DEFAULT;
        for attention in [Attention::Idle, Attention::Near, Attention::Attended] {
            let one = palette.item(attention, false);
            let source = palette.item(attention, true);
            assert_eq!(one, palette.tint(attention), "the one of a thing is plain");
            assert!(
                spread(source) > spread(one),
                "the shift is a warmth, not a lightness"
            );
            assert!(
                spread(source) < spread(palette.accent),
                "and never loud enough to read as engagement"
            );
        }
    }

    #[test]
    fn what_is_in_hand_wears_the_accent_whatever_it_is() {
        let palette = Palette::DEFAULT;
        for source in [true, false] {
            assert_eq!(
                palette.item(Attention::Engaged, source),
                palette.accent,
                "a grasped mote says it is grasped, not what it stocks"
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
