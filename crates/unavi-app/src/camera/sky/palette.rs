//! The colours a sky draws with, all derived from one hue and chroma.

use bevy::prelude::*;
use unavi_space::membership::SpaceId;

/// The most chroma a sky shows, whatever colour it is given, so it never
/// competes with the saturated hues VUI spends on state.
pub const MAX_CHROMA: f32 = 0.05;

/// The part of a colour a sky uses: OKLCH hue in degrees, and chroma.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tone {
    pub hue:    f32,
    pub chroma: f32,
}

impl Tone {
    /// The tone `color` gives a sky: its hue, with chroma held to
    /// [`MAX_CHROMA`]. A grey gives a grey sky.
    pub fn of(color: Color) -> Self {
        let oklch = Oklcha::from(color);
        // Huge but finite channels overflow the conversion, and one NaN would
        // stick in the blend for good.
        if !oklch.hue.is_finite() || !oklch.chroma.is_finite() {
            return Self {
                hue:    0.0,
                chroma: 0.0,
            };
        }
        Self {
            hue:    oklch.hue,
            chroma: oklch.chroma.min(MAX_CHROMA),
        }
    }

    /// The tone of a space that sets no sky: the colour its beacon derives
    /// from the same id.
    pub fn of_space(id: SpaceId) -> Self {
        let [h0, h1, h2, h3, h4, h5, h6, h7, s0, s1, v0, v1, ..] = id.0;
        let hue = u64::from_le_bytes([h0, h1, h2, h3, h4, h5, h6, h7]) as f64 / u64::MAX as f64;
        let unit = |bytes| f32::from(u16::from_le_bytes(bytes)) / f32::from(u16::MAX);
        let saturation = unit([s0, s1]).mul_add(0.25, 0.70);
        let value = unit([v0, v1]).mul_add(0.25, 0.70);

        let rgb = Srgba::from(Hsva::new(hue as f32 * 360.0, saturation, value, 1.0));
        // A beacon hands these channels to a material, which reads them as
        // linear.
        Self::of(Color::linear_rgb(rgb.red, rgb.green, rgb.blue))
    }
}

/// The colours the sky shader and the fog draw with.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub zenith:  LinearRgba,
    pub horizon: LinearRgba,
    /// Added at the horizon and falling off above and below it.
    pub haze:    LinearRgba,
    pub line:    LinearRgba,
    pub glow:    LinearRgba,
}

impl Palette {
    pub fn new(tone: Tone) -> Self {
        let at = |lightness: f32, chroma: f32, hue_shift: f32| {
            let hue = (tone.hue + hue_shift).rem_euclid(360.0);
            let c = LinearRgba::from(Oklcha::new(lightness, tone.chroma * chroma, hue, 1.0));
            // Out-of-gamut tones come back with negative channels.
            LinearRgba::rgb(c.red.max(0.0), c.green.max(0.0), c.blue.max(0.0))
        };
        Self {
            zenith:  at(0.55, 1.0, 0.0),
            horizon: at(0.80, 0.44, -25.0),
            haze:    at(0.92, 0.7, -25.0) * 0.22,
            line:    at(0.88, 1.6, 0.0),
            glow:    at(0.95, 1.3, 0.0),
        }
    }

    /// The sky's colour at the horizon, which fog must fade into to meet it.
    pub fn fog(&self) -> LinearRgba {
        (self.horizon + self.haze).with_alpha(1.0)
    }
}
