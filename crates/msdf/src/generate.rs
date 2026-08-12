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

use crate::atlas::Rect;

/// Untouched texels between neighbours, so a bilinear fetch at the edge of one
/// glyph never picks up the distance field of the next.
pub const GUTTER: i32 = 2;

#[derive(Debug, Clone)]
pub struct GenerateOpts {
    /// Texels per em. Sets how much of the atlas each glyph costs, and the
    /// smallest feature the field can still resolve.
    pub px_per_em:       u32,
    /// Texels the distance field spans either side of an edge. Sets how far
    /// an outline or glow can reach before it runs out of gradient.
    pub range:           f64,
    /// Corner sharpness below which an edge keeps its neighbour's colour.
    pub angle_threshold: f64,
}

impl Default for GenerateOpts {
    fn default() -> Self {
        Self {
            px_per_em:       32,
            range:           6.0,
            angle_threshold: 0.03,
        }
    }
}

pub struct Rendered {
    pub ch:      char,
    pub plane:   Rect,
    pub advance: f32,
    pub field:   Option<RgbaImage>,
}

#[must_use]
pub fn render(face: &Face, id: GlyphId, ch: char, upem: f64, opts: &GenerateOpts) -> Rendered {
    let shrinkage = upem / f64::from(opts.px_per_em);
    let advance = f64::from(face.glyph_hor_advance(id).unwrap_or_default()) / upem;
    let blank = Rendered {
        ch,
        plane: Rect::ZERO,
        advance: advance as f32,
        field: None,
    };

    // A space has an advance and no outline; so does any glyph whose contours
    // the face declines to give us.
    let (Some(bounds), Some(mut shape)) = (
        face.glyph_bounding_box(id),
        fdsm_ttf_parser::load_shape_from_face(face, id),
    ) else {
        return blank;
    };

    let (x_min, y_min) = (f64::from(bounds.x_min), f64::from(bounds.y_min));
    let span = |min: i16, max: i16| {
        2.0f64.mul_add(opts.range, (f64::from(max) - f64::from(min)) / shrinkage)
    };
    let width = span(bounds.x_min, bounds.x_max).ceil() as u32;
    let height = span(bounds.y_min, bounds.y_max).ceil() as u32;
    if width == 0 || height == 0 {
        return blank;
    }

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
