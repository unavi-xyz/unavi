//! A face with its metrics extracted: the font half of runtime generation.
//! The glyph store half lives in `runtime`.

use std::sync::Arc;

use ttf_parser::{
    Face,
    GlyphId,
};

use crate::atlas::VerticalMetrics;

pub struct Font {
    bytes: Arc<[u8]>,
    pub vertical: VerticalMetrics,
    upem: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("parse font: {0}")]
    Face(#[from] ttf_parser::FaceParsingError),
    #[error("font declares no units per em")]
    NoUnitsPerEm,
}

impl Font {
    pub fn parse(bytes: Arc<[u8]>) -> Result<Self, FontError> {
        let face = Face::parse(&bytes, 0)?;
        let upem = f32::from(face.units_per_em());
        if upem <= 0.0 {
            return Err(FontError::NoUnitsPerEm);
        }
        let vertical = {
            let face = Face::parse(&bytes, 0)?;
            VerticalMetrics {
                ascender:  f32::from(face.ascender()) / upem,
                descender: f32::from(face.descender()) / upem,
                line_gap:  f32::from(face.line_gap()) / upem,
            }
        };
        Ok(Self {
            bytes,
            vertical,
            upem,
        })
    }

    #[must_use]
    pub const fn units_per_em(&self) -> f32 {
        self.upem
    }

    /// The face's glyph for a character; `None` means the face itself lacks
    /// it, so no runtime atlas could serve it either.
    #[must_use]
    pub fn glyph_index(&self, ch: char) -> Option<GlyphId> {
        Face::parse(&self.bytes, 0).ok()?.glyph_index(ch)
    }

    /// Borrows the face for one call. Cheap enough to do per use; the bytes
    /// are immutable, so a face that parsed once parses again.
    #[must_use]
    pub fn with_face<T>(&self, f: impl FnOnce(&Face) -> T) -> Option<T> {
        Some(f(&Face::parse(&self.bytes, 0).ok()?))
    }

    /// Pair adjustment, shaped on demand. Latin stays in the resident baked
    /// page's own kern map; this is for generated glyphs, where a pair that
    /// kerns is the exception rather than the rule.
    #[must_use]
    pub fn kern(&self, left: char, right: char) -> f32 {
        let Ok(face) = harfrust::FontRef::new(&self.bytes) else {
            return 0.0;
        };
        let data = harfrust::ShaperData::new(&face);
        let shaper = data.shaper(&face).build();

        let shape = |text: &str| {
            let mut unicode = harfrust::UnicodeBuffer::new();
            unicode.push_str(text);
            unicode.guess_segment_properties();
            shaper.shape(unicode, &[])
        };
        let width = |result: &harfrust::GlyphBuffer| {
            result
                .glyph_positions()
                .iter()
                .map(|pos| pos.x_advance)
                .sum::<i32>()
        };

        let mut pair = String::with_capacity(8);
        pair.push(left);
        pair.push(right);
        let whole = shape(&pair);
        // A pair that shaped into one glyph is a ligature, not a kern.
        if whole.glyph_infos().len() != 2 {
            return 0.0;
        }
        let mut alone = [0u8; 4];
        let left_width = width(&shape(left.encode_utf8(&mut alone)));
        let right_width = width(&shape(right.encode_utf8(&mut alone)));
        (width(&whole) - left_width - right_width) as f32 / self.upem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> Font {
        Font::parse(Arc::<[u8]>::from(notosans::REGULAR_TTF)).expect("parse")
    }

    #[test]
    fn metrics_describe_a_usable_line() {
        let font = font();
        assert!(font.vertical.ascender > 0.0);
        assert!(font.vertical.descender < 0.0);
        assert!(font.vertical.line_height() > 1.0);
    }

    #[test]
    fn a_face_that_has_a_glyph_says_so() {
        let font = font();
        assert!(font.glyph_index('A').is_some());
        assert!(
            font.glyph_index('漢').is_none(),
            "Noto Sans has no CJK glyphs"
        );
    }

    #[test]
    fn a_gpos_pair_is_found_on_demand() {
        let font = font();
        assert!(
            font.kern('A', 'V') < 0.0,
            "GPOS pair adjustments must survive outside the bake"
        );
        assert!(font.kern('x', 'y').abs() < 1.0e-6);
    }
}
