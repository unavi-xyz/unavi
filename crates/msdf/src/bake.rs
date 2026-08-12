use std::collections::BTreeMap;

use etagere::{
    AtlasAllocator,
    size2,
};
use image::RgbaImage;
use ttf_parser::Face;

use crate::atlas::{
    Atlas,
    Glyph,
    Rect,
    VerticalMetrics,
};
use crate::generate::{
    GenerateOpts,
    GUTTER,
    Rendered,
};

/// Noto Sans Regular, SIL Open Font License 1.1; the licence text travels with
/// the `notosans` crate.
pub const DEFAULT_FONT: &[u8] = notosans::REGULAR_TTF;

const FIRST_SIZE: u32 = 256;
const MAX_SIZE: u32 = 4096;

#[derive(Debug, Clone)]
pub struct BakeOpts {
    /// Texels per em. Sets how much of the atlas each glyph costs, and the
    /// smallest feature the field can still resolve.
    pub px_per_em:       u32,
    /// Texels the distance field spans either side of an edge. Sets how far
    /// an outline or glow can reach before it runs out of gradient.
    pub range:           f64,
    /// Corner sharpness below which an edge keeps its neighbour's colour.
    pub angle_threshold: f64,
    pub charset:         String,
}

impl Default for BakeOpts {
    fn default() -> Self {
        Self {
            px_per_em:       32,
            range:           6.0,
            angle_threshold: 0.03,
            charset:         crate::atlas::LATIN.to_string(),
        }
    }
}

impl From<&BakeOpts> for GenerateOpts {
    fn from(opts: &BakeOpts) -> Self {
        Self {
            px_per_em:       opts.px_per_em,
            range:           opts.range,
            angle_threshold: opts.angle_threshold,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BakeError {
    #[error("parse font: {0}")]
    Face(#[from] ttf_parser::FaceParsingError),
    #[error("font declares no units per em")]
    NoUnitsPerEm,
    #[error("{glyphs} glyphs do not fit in a {MAX_SIZE}x{MAX_SIZE} atlas")]
    TooLarge { glyphs: usize },
}

pub struct Baked {
    pub atlas:   Atlas,
    pub image:   RgbaImage,
    /// Characters the face has no glyph for; the answer to a missing one is to
    /// change the charset or the font.
    pub missing: Vec<char>,
}

pub fn bake(font: &[u8], opts: &BakeOpts) -> Result<Baked, BakeError> {
    let face = Face::parse(font, 0)?;
    let upem = f64::from(face.units_per_em());
    if upem <= 0.0 {
        return Err(BakeError::NoUnitsPerEm);
    }

    let mut charset = opts.charset.chars().collect::<Vec<_>>();
    charset.sort_unstable();
    charset.dedup();

    let generate = GenerateOpts::from(opts);
    let mut rendered = Vec::with_capacity(charset.len());
    let mut missing = Vec::new();

    for ch in &charset {
        let Some(id) = face.glyph_index(*ch) else {
            missing.push(*ch);
            continue;
        };
        rendered.push(crate::generate::render(&face, id, *ch, upem, &generate));
    }

    let (image, placed) = pack(&rendered)?;
    let (width, height) = (image.width(), image.height());

    let glyphs = rendered
        .iter()
        .zip(placed)
        .map(|(glyph, uv)| {
            (
                glyph.ch,
                Glyph {
                    plane: glyph.plane,
                    uv,
                    advance: glyph.advance,
                    page: 0,
                    font: 0,
                },
            )
        })
        .collect();

    Ok(Baked {
        atlas: Atlas {
            width,
            height,
            range: opts.range as f32,
            vertical: VerticalMetrics {
                ascender:  f32::from(face.ascender()) / upem as f32,
                descender: f32::from(face.descender()) / upem as f32,
                line_gap:  f32::from(face.line_gap()) / upem as f32,
            },
            glyphs,
            kerning: kerning(font, &charset, upem as f32),
        },
        image,
        missing,
    })
}

/// Places every field in one image, growing until they fit. Fields are blitted
/// upside down: font space is y-up and an image is y-down.
fn pack(rendered: &[Rendered]) -> Result<(RgbaImage, Vec<Rect>), BakeError> {
    let mut size = FIRST_SIZE;

    loop {
        let mut allocator = AtlasAllocator::new(size2(size.cast_signed(), size.cast_signed()));
        let mut image = RgbaImage::new(size, size);
        let mut uvs = Vec::with_capacity(rendered.len());
        let mut fit = true;

        for glyph in rendered {
            let Some(field) = &glyph.field else {
                uvs.push(Rect::ZERO);
                continue;
            };
            let request = size2(
                field.width().cast_signed() + GUTTER,
                field.height().cast_signed() + GUTTER,
            );
            let Some(slot) = allocator.allocate(request) else {
                fit = false;
                break;
            };

            let (x, y) = (slot.rectangle.min.x as u32, slot.rectangle.min.y as u32);
            for (column, row, pixel) in field.enumerate_pixels() {
                let flipped = field.height() - 1 - row;
                image.put_pixel(x + column, y + flipped, *pixel);
            }

            let texels = size as f32;
            uvs.push(Rect {
                min: [x as f32 / texels, y as f32 / texels],
                max: [
                    (x + field.width()) as f32 / texels,
                    (y + field.height()) as f32 / texels,
                ],
            });
        }

        if fit {
            return Ok((image, uvs));
        }
        size *= 2;
        if size > MAX_SIZE {
            return Err(BakeError::TooLarge {
                glyphs: rendered.len(),
            });
        }
    }
}

/// Pair adjustments, measured by shaping each pair. A shaper, not the legacy
/// `kern` table: modern faces keep pair positioning in GPOS.
fn kerning(font: &[u8], charset: &[char], upem: f32) -> BTreeMap<(char, char), f32> {
    let Ok(face) = harfrust::FontRef::new(font) else {
        return BTreeMap::new();
    };
    let data = harfrust::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();

    let mut buffer = Some(harfrust::UnicodeBuffer::new());
    let mut advance = |text: &str, glyphs: usize| {
        let mut unicode = buffer.take().unwrap_or_default();
        unicode.push_str(text);
        unicode.guess_segment_properties();
        let result = shaper.shape(unicode, &[]);
        // A pair that shaped into one glyph is a ligature, not a kern.
        let total = (result.len() == glyphs).then(|| {
            result
                .glyph_positions()
                .iter()
                .map(|pos| pos.x_advance)
                .sum()
        });
        buffer = Some(result.clear());
        total
    };

    let singles = charset
        .iter()
        .filter_map(|ch| {
            let mut text = [0u8; 4];
            advance(ch.encode_utf8(&mut text), 1).map(|width: i32| (*ch, width))
        })
        .collect::<Vec<_>>();

    let mut pairs = BTreeMap::new();
    let mut text = String::with_capacity(8);
    for (left, left_width) in &singles {
        for (right, right_width) in &singles {
            text.clear();
            text.push(*left);
            text.push(*right);
            let Some(result) = advance(&text, 2) else {
                continue;
            };
            let adjustment = result - left_width - right_width;
            if adjustment != 0 {
                pairs.insert((*left, *right), adjustment as f32 / upem);
            }
        }
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atlas::GlyphSource;

    fn baked(charset: &str) -> Baked {
        bake(
            notosans::REGULAR_TTF,
            &BakeOpts {
                charset: charset.to_string(),
                ..Default::default()
            },
        )
        .expect("bake")
    }

    #[test]
    fn every_requested_character_gets_a_glyph() {
        let baked = baked("Hello, world!");
        assert!(baked.missing.is_empty());
        for ch in "Helo,wrd!".chars() {
            let glyph = baked.atlas.glyph(ch).expect("glyph");
            assert!(glyph.advance > 0.0, "{ch} advances");
            assert!(!glyph.plane.is_empty(), "{ch} has a field");
        }
    }

    #[test]
    fn a_space_advances_without_a_field() {
        let baked = baked("a b");
        let space = baked.atlas.glyph(' ').expect("space");
        assert!(space.advance > 0.0);
        assert!(space.plane.is_empty(), "a space is not drawn");
        assert!(space.uv.is_empty(), "and takes no room in the atlas");
    }

    #[test]
    fn every_glyph_lands_inside_the_atlas() {
        let baked = baked(crate::atlas::LATIN);
        for (ch, glyph) in &baked.atlas.glyphs {
            for corner in [glyph.uv.min, glyph.uv.max] {
                assert!(
                    (0.0..=1.0).contains(&corner[0]) && (0.0..=1.0).contains(&corner[1]),
                    "{ch} at {corner:?}"
                );
            }
        }
    }

    #[test]
    fn the_field_carries_a_distance_gradient() {
        let baked = baked("O");
        let partial = baked
            .image
            .pixels()
            .filter(|pixel| (1..255).contains(&pixel.0[3]))
            .count();
        assert!(partial > 0, "a field of hard 0s and 255s is a bitmap");
    }

    #[test]
    fn a_cornered_glyph_uses_more_than_one_channel() {
        let baked = baked("M");
        let varied = baked
            .image
            .pixels()
            .filter(|pixel| pixel.0[0] != pixel.0[1] || pixel.0[1] != pixel.0[2])
            .count();
        assert!(
            varied > 0,
            "corners are what the extra channels exist to keep sharp; agreeing \
             channels mean the edge colouring never ran"
        );
    }

    #[test]
    fn the_plane_sits_on_the_baseline() {
        let baked = baked("xX");
        let (small, large) = (
            baked.atlas.glyph('x').expect("x"),
            baked.atlas.glyph('X').expect("X"),
        );
        assert!(
            large.plane.max[1] > small.plane.max[1],
            "a cap is taller than an x from the same baseline"
        );
    }

    #[test]
    fn gpos_pair_positioning_is_found() {
        let baked = baked("AVTo");
        assert!(
            baked.atlas.kern('A', 'V') < 0.0,
            "the face kerns this pair through GPOS, which no `kern` table holds"
        );
    }

    #[test]
    fn vertical_metrics_describe_a_usable_line() {
        let baked = baked("a");
        let vertical = baked.atlas.vertical;
        assert!(vertical.ascender > 0.0);
        assert!(vertical.descender < 0.0);
        assert!(vertical.line_height() > 1.0);
    }
}
