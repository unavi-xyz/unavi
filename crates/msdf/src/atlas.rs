use std::collections::BTreeMap;

use serde::{
    Deserialize,
    Serialize,
};

/// An axis-aligned box; a glyph is a pair, one in em units and one in texture
/// coordinates.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl Rect {
    pub const ZERO: Self = Self {
        min: [0.0, 0.0],
        max: [0.0, 0.0],
    };

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.max[0] <= self.min[0] || self.max[1] <= self.min[1]
    }

    #[must_use]
    pub fn translated(&self, x: f32, y: f32) -> Self {
        Self {
            min: [self.min[0] + x, self.min[1] + y],
            max: [self.max[0] + x, self.max[1] + y],
        }
    }

    #[must_use]
    pub fn scaled(&self, factor: f32) -> Self {
        Self {
            min: [self.min[0] * factor, self.min[1] * factor],
            max: [self.max[0] * factor, self.max[1] * factor],
        }
    }

    pub const fn union(&mut self, other: &Self) {
        self.min[0] = self.min[0].min(other.min[0]);
        self.min[1] = self.min[1].min(other.min[1]);
        self.max[0] = self.max[0].max(other.max[0]);
        self.max[1] = self.max[1].max(other.max[1]);
    }
}

/// One glyph's placement, in em units so a caller picks the size.
///
/// `plane` sits on the baseline with the pen at the origin, y up. `uv.min` is
/// the top-left texel, since the atlas is stored in image order while the
/// plane is in font order.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Glyph {
    pub plane:   Rect,
    pub uv:      Rect,
    pub advance: f32,
    /// Which page of a multi-page atlas the field lives in; a baked single
    /// image is always page 0.
    pub page:    u32,
    /// Which font in a fallback stack the glyph came from; a single-font
    /// source always stamps 0.
    #[serde(default)]
    pub font:    u32,
}

/// Line spacing in em units.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VerticalMetrics {
    pub ascender:  f32,
    pub descender: f32,
    pub line_gap:  f32,
}

impl VerticalMetrics {
    /// Baseline-to-baseline distance at one em.
    #[must_use]
    pub fn line_height(&self) -> f32 {
        self.ascender - self.descender + self.line_gap
    }
}

/// A baked font: one multi-channel distance field, and everything needed to
/// lay text out against it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Atlas {
    pub width:    u32,
    pub height:   u32,
    /// Texels the signed distance spans. Baked and drawn ranges must match, or
    /// the edge is soft or aliased.
    pub range:    f32,
    pub vertical: VerticalMetrics,
    pub glyphs:   BTreeMap<char, Glyph>,
    pub kerning:  BTreeMap<(char, char), f32>,
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("postcard {0}")]
    Postcard(#[from] postcard::Error),
}

impl Atlas {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        Ok(postcard::from_bytes(bytes)?)
    }

    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_stdvec(self)
    }
}

/// What laying text out needs from a glyph store: vertical metrics, per-char
/// placement, and pair adjustments. The baked [`Atlas`] and the runtime atlas
/// both satisfy it.
pub trait GlyphSource {
    fn vertical(&self) -> VerticalMetrics;
    fn glyph(&self, ch: char) -> Option<Glyph>;
    fn kern(&self, left: char, right: char) -> f32;
    /// Whether `glyph(ch)` fell back to a placeholder instead of the font's
    /// real glyph. Layout reports these characters so a caller can ask the
    /// source for them.
    fn missing(&self, ch: char) -> bool {
        self.glyph(ch).is_none()
    }
}

impl GlyphSource for Atlas {
    fn vertical(&self) -> VerticalMetrics {
        self.vertical
    }

    fn glyph(&self, ch: char) -> Option<Glyph> {
        self.glyphs.get(&ch).copied()
    }

    fn kern(&self, left: char, right: char) -> f32 {
        self.kerning.get(&(left, right)).copied().unwrap_or(0.0)
    }
}

/// Printable ASCII, the Latin-1 letters, and the punctuation a UI reaches for.
///
/// Wider coverage is a separate atlas: CJK at a legible texel density is too
/// large to embed.
pub const LATIN: &str = concat!(
    " !\"#$%&'()*+,-./0123456789:;<=>?@",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`",
    "abcdefghijklmnopqrstuvwxyz{|}~",
    "¡¢£¥§©«®°±·»¿×÷",
    "ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÑÒÓÔÕÖØÙÚÛÜÝß",
    "àáâãäåæçèéêëìíîïñòóôõöøùúûüýÿ",
    "–—‘’‚“”„†•…‰€™←→↑↓",
);

#[cfg(test)]
mod tests {
    use super::*;

    fn atlas() -> Atlas {
        Atlas {
            width:    64,
            height:   64,
            range:    4.0,
            vertical: VerticalMetrics {
                ascender:  0.8,
                descender: -0.2,
                line_gap:  0.1,
            },
            glyphs:   BTreeMap::from([(
                'a',
                Glyph {
                    plane:   Rect {
                        min: [0.0, 0.0],
                        max: [0.5, 0.5],
                    },
                    uv:      Rect {
                        min: [0.0, 0.0],
                        max: [0.25, 0.25],
                    },
                    advance: 0.6,
                    page:    0,
                    font:    0,
                },
            )]),
            kerning:  BTreeMap::from([(('A', 'V'), -0.08)]),
        }
    }

    #[test]
    fn an_atlas_survives_a_round_trip() {
        let atlas = atlas();
        let bytes = atlas.encode().expect("encode");
        assert_eq!(Atlas::decode(&bytes).expect("decode"), atlas);
    }

    #[test]
    fn an_unkerned_pair_costs_nothing() {
        assert!((atlas().kern('A', 'V') + 0.08).abs() < 1.0e-6);
        assert!(atlas().kern('x', 'y').abs() < 1.0e-6);
    }

    #[test]
    fn line_height_spans_ascender_to_descender_plus_the_gap() {
        assert!((atlas().vertical.line_height() - 1.1).abs() < 1.0e-6);
    }

    #[test]
    fn a_zero_area_rect_is_empty() {
        assert!(Rect::ZERO.is_empty());
        assert!(!atlas().glyphs[&'a'].plane.is_empty());
    }

    #[test]
    fn the_latin_charset_holds_no_duplicates() {
        let mut seen = LATIN.chars().collect::<Vec<_>>();
        let total = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), total);
    }
}
