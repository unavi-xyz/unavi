//! A face parsed once and kept: the font half of runtime generation. The
//! glyph store half lives in `runtime`.
//!
//! Every lookup here is on a hot path — layout asks for a glyph index and a
//! pair adjustment per character of every string, every frame it rebuilds — so
//! the parsed tables and the shaper are built at construction and the pair
//! adjustments are memoized.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        Mutex,
    },
};

use harfrust::{
    FontRef,
    GlyphBuffer,
    ShaperData,
    UnicodeBuffer,
};
use self_cell::self_cell;
use ttf_parser::{
    Face,
    GlyphId,
};

use crate::atlas::VerticalMetrics;

/// Pair adjustments remembered at once. Text is untrusted and a pair is any
/// two characters, so the memo is emptied rather than grown once it fills.
const MAX_KERN_PAIRS: usize = 8192;

struct Tables<'a> {
    face:   Face<'a>,
    shaper: ShaperData,
    font:   FontRef<'a>,
}

self_cell!(
    struct Parsed {
        owner:     Arc<[u8]>,
        #[covariant]
        dependent: Tables,
    }
);

struct Shaping<'a>(harfrust::Shaper<'a>);

self_cell!(
    struct Shaper {
        owner:     Parsed,
        #[not_covariant]
        dependent: Shaping,
    }
);

pub struct Font {
    shaper:       Shaper,
    pub vertical: VerticalMetrics,
    upem:         f32,
    kerns:        Mutex<HashMap<(char, char), f32>>,
}

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("parse font: {0}")]
    Face(#[from] ttf_parser::FaceParsingError),
    #[error("font declares no units per em")]
    NoUnitsPerEm,
    #[error("font is not shapeable")]
    NotShapeable,
    #[error("{0} bytes exceeds the {1} byte cap")]
    TooLarge(usize, usize),
}

/// Bytes one face may occupy. A CJK face is tens of megabytes; past this a
/// file is not a font a client should be holding in memory.
pub const MAX_FONT_BYTES: usize = 64 * 1024 * 1024;

impl Font {
    pub fn parse(bytes: Arc<[u8]>) -> Result<Self, FontError> {
        if bytes.len() > MAX_FONT_BYTES {
            return Err(FontError::TooLarge(bytes.len(), MAX_FONT_BYTES));
        }
        let parsed = Parsed::try_new(bytes, |bytes| -> Result<Tables<'_>, FontError> {
            let face = Face::parse(bytes, 0)?;
            let font = FontRef::new(bytes).map_err(|_| FontError::NotShapeable)?;
            Ok(Tables {
                face,
                shaper: ShaperData::new(&font),
                font,
            })
        })?;

        let upem = f32::from(parsed.borrow_dependent().face.units_per_em());
        if upem <= 0.0 {
            return Err(FontError::NoUnitsPerEm);
        }
        let vertical = {
            let face = &parsed.borrow_dependent().face;
            VerticalMetrics {
                ascender:  f32::from(face.ascender()) / upem,
                descender: f32::from(face.descender()) / upem,
                line_gap:  f32::from(face.line_gap()) / upem,
            }
        };

        Ok(Self {
            shaper: Shaper::new(parsed, |parsed| {
                let tables = parsed.borrow_dependent();
                Shaping(tables.shaper.shaper(&tables.font).build())
            }),
            vertical,
            upem,
            kerns: Mutex::new(HashMap::new()),
        })
    }

    #[must_use]
    pub const fn units_per_em(&self) -> f32 {
        self.upem
    }

    #[must_use]
    pub fn face(&self) -> &Face<'_> {
        &self.shaper.borrow_owner().borrow_dependent().face
    }

    /// The face's glyph for a character; `None` means the face itself lacks
    /// it, so no runtime atlas could serve it either.
    #[must_use]
    pub fn glyph_index(&self, ch: char) -> Option<GlyphId> {
        self.face().glyph_index(ch)
    }

    /// The character's advance in em units, whether or not it has an outline.
    #[must_use]
    pub fn advance(&self, ch: char) -> Option<f32> {
        let face = self.face();
        let id = face.glyph_index(ch)?;
        Some(f32::from(face.glyph_hor_advance(id).unwrap_or_default()) / self.upem)
    }

    /// Pair adjustment, shaped on demand and remembered. A pair that kerns is
    /// the exception rather than the rule, so most of what this stores is
    /// zeroes — cheaper than reshaping the same pair every frame.
    #[must_use]
    pub fn kern(&self, left: char, right: char) -> f32 {
        if let Ok(kerns) = self.kerns.lock()
            && let Some(kern) = kerns.get(&(left, right))
        {
            return *kern;
        }
        let kern = self.shape_kern(left, right);
        if let Ok(mut kerns) = self.kerns.lock() {
            if kerns.len() >= MAX_KERN_PAIRS {
                kerns.clear();
            }
            kerns.insert((left, right), kern);
        }
        kern
    }

    fn shape_kern(&self, left: char, right: char) -> f32 {
        self.shaper.with_dependent(|_, shaping| {
            let shape = |text: &str| {
                let mut unicode = UnicodeBuffer::new();
                unicode.push_str(text);
                unicode.guess_segment_properties();
                shaping.0.shape(unicode, &[])
            };
            let width = |result: &GlyphBuffer| {
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
        })
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

    #[test]
    fn a_repeated_pair_answers_from_the_memo() {
        let font = font();
        let first = font.kern('A', 'V');
        assert!((font.kern('A', 'V') - first).abs() < 1.0e-6);
        assert_eq!(font.kerns.lock().expect("kerns").len(), 1);
    }

    #[test]
    fn the_memo_never_grows_past_its_cap() {
        let font = font();
        for (index, left) in ('\u{0}'..).take(MAX_KERN_PAIRS + 16).enumerate() {
            let _ = font.kern(left, char::from_u32(index as u32).unwrap_or('a'));
        }
        assert!(font.kerns.lock().expect("kerns").len() <= MAX_KERN_PAIRS);
    }

    #[test]
    fn an_advance_comes_back_without_an_outline() {
        let font = font();
        assert!(
            font.advance(' ').unwrap_or_default() > 0.0,
            "a space is wide"
        );
        assert!(font.advance('漢').is_none(), "the face has no such glyph");
    }

    #[test]
    fn garbage_bytes_are_an_error_rather_than_a_panic() {
        assert!(Font::parse(Arc::<[u8]>::from(&b"not a font at all"[..])).is_err());
        assert!(Font::parse(Arc::<[u8]>::from(&[][..])).is_err());
    }

    #[test]
    fn an_oversized_file_is_refused_before_it_is_parsed() {
        let bytes = Arc::<[u8]>::from(vec![0u8; MAX_FONT_BYTES + 1]);
        assert!(matches!(
            Font::parse(bytes),
            Err(FontError::TooLarge(_, MAX_FONT_BYTES))
        ));
    }
}
