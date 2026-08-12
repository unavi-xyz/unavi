//! Per-glyph multi-channel distance field generation, shared by the bake and
//! the runtime atlas.

use fdsm::{
    bezier::scanline::FillRule,
    generate::generate_mtsdf,
    render::correct_sign_mtsdf,
    shape::Shape,
    transform::Transform,
};
use image::RgbaImage;
use nalgebra::{
    Affine2,
    Similarity2,
    Vector2,
};
use ttf_parser::{
    Face,
    GlyphId,
};

use crate::{
    atlas::Rect,
    outline::{
        self,
        Limits,
    },
};

/// Untouched texels between neighbours, so a bilinear fetch at the edge of one
/// glyph never picks up the distance field of the next.
pub const GUTTER: i32 = 2;

#[derive(Debug, Clone, Copy)]
pub struct GenerateOpts {
    /// Texels per em. Sets how much of the atlas each glyph costs, and the
    /// smallest feature the field can still resolve.
    pub px_per_em:       u32,
    /// Texels the distance field spans either side of an edge. Sets how far
    /// an outline or glow can reach before it runs out of gradient.
    pub range:           f64,
    /// Corner sharpness below which an edge keeps its neighbour's colour.
    pub angle_threshold: f64,
    /// Texels a glyph's field may span on either axis. A face may declare a
    /// bounding box thousands of ems wide; without this the field it asks for
    /// is an allocation nothing can serve.
    pub max_field:       u32,
    pub outline:         Limits,
}

impl Default for GenerateOpts {
    fn default() -> Self {
        Self {
            px_per_em:       32,
            range:           6.0,
            angle_threshold: 0.03,
            max_field:       256,
            outline:         Limits::default(),
        }
    }
}

pub struct Rendered {
    pub ch:      char,
    pub plane:   Rect,
    pub advance: f32,
    pub field:   Option<RgbaImage>,
    /// The glyph's outline was past a cap and no field was generated, so the
    /// character advances without ink. A caller counts these: a face that
    /// trips it is malformed or hostile.
    pub refused: bool,
}

#[must_use]
pub fn render(face: &Face, id: GlyphId, ch: char, upem: f64, opts: &GenerateOpts) -> Rendered {
    let shrinkage = upem / f64::from(opts.px_per_em.max(1));
    let advance = f64::from(face.glyph_hor_advance(id).unwrap_or_default()) / upem;
    let blank = Rendered {
        ch,
        plane: Rect::ZERO,
        advance: advance as f32,
        field: None,
        refused: false,
    };

    // A space has an advance and no outline; so does any glyph whose contours
    // the face declines to give us, or one too complex to sample.
    let Some(bounds) = face.glyph_bounding_box(id) else {
        return blank;
    };
    let Some(mut shape) = outline::load(face, id, opts.outline) else {
        return Rendered {
            refused: true,
            ..blank
        };
    };

    let (x_min, y_min) = (f64::from(bounds.x_min), f64::from(bounds.y_min));
    let span = |min: i16, max: i16| {
        2.0f64.mul_add(opts.range, (f64::from(max) - f64::from(min)) / shrinkage)
    };
    let extent = |span: f64| {
        let texels = span.ceil();
        if texels.is_finite() && (1.0..=f64::from(opts.max_field)).contains(&texels) {
            Some(texels as u32)
        } else {
            None
        }
    };
    let (Some(width), Some(height)) = (
        extent(span(bounds.x_min, bounds.x_max)),
        extent(span(bounds.y_min, bounds.y_max)),
    ) else {
        return Rendered {
            refused: true,
            ..blank
        };
    };

    let transformation = nalgebra::convert::<_, Affine2<f64>>(Similarity2::new(
        Vector2::new(
            opts.range - x_min / shrinkage,
            opts.range - y_min / shrinkage,
        ),
        0.0,
        1.0 / shrinkage,
    ));
    shape.transform(&transformation);
    let shape = Shape::edge_coloring_simple(shape, opts.angle_threshold, 0).prepare();

    let mut field = RgbaImage::new(width, height);
    generate_mtsdf(&shape, opts.range, &mut field);
    correct_sign_mtsdf(&mut field, &shape, FillRule::Nonzero);

    // The field covers whole texels, so its extent comes back from the pixel
    // count rather than from the bounding box the ceil rounded up from.
    let edge = |min: f64, texels: u32| {
        [
            opts.range.mul_add(-shrinkage, min) / upem,
            (f64::from(texels) - opts.range).mul_add(shrinkage, min) / upem,
        ]
    };
    let [left, right] = edge(x_min, width);
    let [bottom, top] = edge(y_min, height);

    Rendered {
        plane: Rect {
            min: [left as f32, bottom as f32],
            max: [right as f32, top as f32],
        },
        field: Some(field),
        ..blank
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::font::Font;

    fn font() -> Font {
        Font::parse(Arc::<[u8]>::from(notosans::REGULAR_TTF)).expect("parse")
    }

    fn rendered(ch: char, opts: &GenerateOpts) -> Rendered {
        let font = font();
        let id = font.glyph_index(ch).expect("glyph");
        render(font.face(), id, ch, f64::from(font.units_per_em()), opts)
    }

    #[test]
    fn a_letter_renders_a_field_around_its_outline() {
        let rendered = rendered('A', &GenerateOpts::default());
        let field = rendered.field.expect("field");
        assert!(field.width() > 0 && field.height() > 0);
        assert!(!rendered.refused);
        assert!(rendered.plane.max[1] > 0.0 && rendered.advance > 0.0);
    }

    #[test]
    fn a_field_wider_than_the_cap_is_refused_rather_than_allocated() {
        let rendered = rendered(
            'A',
            &GenerateOpts {
                max_field: 4,
                ..Default::default()
            },
        );
        assert!(rendered.field.is_none());
        assert!(rendered.refused, "the caller can count what it refused");
        assert!(rendered.advance > 0.0, "the character still advances");
    }

    #[test]
    fn a_glyph_past_the_segment_cap_is_refused() {
        let rendered = rendered(
            '@',
            &GenerateOpts {
                outline: Limits { segments: 2 },
                ..Default::default()
            },
        );
        assert!(rendered.field.is_none());
        assert!(rendered.refused);
    }

    #[test]
    fn a_space_is_blank_without_being_refused() {
        let rendered = rendered(' ', &GenerateOpts::default());
        assert!(rendered.field.is_none());
        assert!(!rendered.refused, "a space is not a malformed glyph");
        assert!(rendered.advance > 0.0);
    }

    #[test]
    fn a_zero_density_never_divides_by_zero() {
        let rendered = rendered(
            'A',
            &GenerateOpts {
                px_per_em: 0,
                ..Default::default()
            },
        );
        assert!(rendered.advance.is_finite());
    }
}
