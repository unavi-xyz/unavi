//! The ambient light's colour: the shade token, leaning toward the sky's hue.

use bevy::prelude::*;

use super::{
    blend::SkyBlend,
    palette::Tone,
};
use crate::style::{
    FIELD_SHADE_CHROMA,
    FIELD_SHADE_HUE,
    FIELD_SKY_CHROMA_MAX,
};

/// The most degrees the shade hue turns toward the sky's.
const MAX_LEAN: f32 = 30.0;
/// The fraction of the hue gap the shade closes, before [`MAX_LEAN`] applies.
const LEAN: f32 = 0.25;
/// OKLCH lightness of the ambient colour. Its brightest channel is then
/// scaled to 1, so the tint changes the hue of the ambient light and not its
/// brightness.
const LIGHTNESS: f32 = 0.9;

/// The tone the shade takes under a sky of `sky`. Its chroma is always the
/// token's, and its hue stays within [`MAX_LEAN`] of the token's. A grey sky
/// gives the token itself, and the lean grows with the sky's chroma.
#[must_use]
pub fn shade_tone(sky: Tone) -> Tone {
    let gap = (sky.hue - FIELD_SHADE_HUE + 180.0).rem_euclid(360.0) - 180.0;
    let weight = (sky.chroma / FIELD_SKY_CHROMA_MAX).clamp(0.0, 1.0);
    let lean = (gap * LEAN * weight).clamp(-MAX_LEAN, MAX_LEAN);
    Tone {
        hue:    (FIELD_SHADE_HUE + lean).rem_euclid(360.0),
        chroma: FIELD_SHADE_CHROMA,
    }
}

/// The ambient colour for `tone`: near white, with its brightest channel 1.
#[must_use]
pub fn ambient_color(tone: Tone) -> Color {
    let rgb = LinearRgba::from(Oklcha::new(LIGHTNESS, tone.chroma, tone.hue, 1.0));
    // Out-of-gamut tones come back with negative channels.
    let [red, green, blue] = [rgb.red, rgb.green, rgb.blue].map(|channel| channel.max(0.0));
    let peak = red.max(green).max(blue).max(f32::EPSILON);
    Color::linear_rgb(red / peak, green / peak, blue / peak)
}

/// Tints the global ambient light by the sky as currently drawn, which is
/// already eased, so the shade follows it without a spring of its own.
pub fn tint_ambient(sky: Res<SkyBlend>, mut ambient: ResMut<GlobalAmbientLight>) {
    ambient.color = ambient_color(shade_tone(sky.tone()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hue_gap(a: f32, b: f32) -> f32 {
        ((a - b + 180.0).rem_euclid(360.0) - 180.0).abs()
    }

    #[test]
    fn a_grey_sky_leaves_the_shade_at_the_token() {
        let tone = shade_tone(Tone {
            hue:    123.0,
            chroma: 0.0,
        });
        assert!(hue_gap(tone.hue, FIELD_SHADE_HUE) < 1.0e-3);
        assert!((tone.chroma - FIELD_SHADE_CHROMA).abs() < f32::EPSILON);
    }

    #[test]
    fn a_sky_at_the_shade_hue_leaves_the_shade_at_the_token() {
        let tone = shade_tone(Tone {
            hue:    FIELD_SHADE_HUE,
            chroma: FIELD_SKY_CHROMA_MAX,
        });
        assert!(hue_gap(tone.hue, FIELD_SHADE_HUE) < 1.0e-3);
    }

    #[test]
    fn the_shade_never_has_more_chroma_than_the_token() {
        for step in 0..=36 {
            for chroma in [0.0, 0.02, FIELD_SKY_CHROMA_MAX, 1.0] {
                let hue = step as f32 * 10.0;
                assert!(shade_tone(Tone { hue, chroma }).chroma <= FIELD_SHADE_CHROMA);
            }
        }
    }

    #[test]
    fn the_hue_turns_toward_the_sky_by_a_bounded_amount() {
        for step in 0..=36 {
            let hue = step as f32 * 10.0;
            let tone = shade_tone(Tone {
                hue,
                chroma: FIELD_SKY_CHROMA_MAX,
            });
            let turned = hue_gap(tone.hue, FIELD_SHADE_HUE);
            assert!(
                turned <= MAX_LEAN + 1.0e-3,
                "sky {hue} turned the shade {turned}"
            );
            if hue_gap(hue, FIELD_SHADE_HUE) > 1.0 {
                assert!(
                    hue_gap(tone.hue, hue) < hue_gap(FIELD_SHADE_HUE, hue),
                    "sky {hue} did not pull the shade closer"
                );
            }
        }
    }

    #[test]
    fn the_ambient_stays_near_white() {
        for step in 0..=36 {
            let hue = step as f32 * 10.0;
            let color = LinearRgba::from(ambient_color(Tone {
                hue,
                chroma: FIELD_SHADE_CHROMA,
            }));
            let channels = [color.red, color.green, color.blue];
            let peak = channels.iter().copied().fold(0.0, f32::max);
            let floor = channels.iter().copied().fold(1.0, f32::min);
            assert!((peak - 1.0).abs() < 1.0e-4, "hue {hue} peaked at {peak}");
            assert!(floor >= 0.6, "hue {hue} dropped a channel to {floor}");
        }
    }
}
