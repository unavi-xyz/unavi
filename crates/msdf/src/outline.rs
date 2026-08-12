//! A glyph outline as bezier contours.
//!
//! Fonts are untrusted: a face may hand back a curve before it opens a
//! contour, or a glyph made of a hundred thousand segments whose field would
//! take minutes to sample. Neither may panic or run long, so a stray segment
//! is dropped and a glyph past [`Limits::segments`] is refused whole.

use fdsm::{
    bezier::{
        Point,
        Segment,
    },
    shape::{
        Contour,
        Shape,
    },
};
use ttf_parser::{
    Face,
    GlyphId,
    OutlineBuilder,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Segments one glyph may hold. Every texel of the field measures its
    /// distance to every segment, so this bounds generation time.
    pub segments: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self { segments: 4096 }
    }
}

/// `None` when the face has no outline for the glyph, or when the outline is
/// past `limits`.
#[must_use]
pub fn load(face: &Face, id: GlyphId, limits: Limits) -> Option<Shape<Contour>> {
    let mut builder = Builder {
        shape: Shape::default(),
        start: None,
        last: None,
        count: 0,
        limits,
    };
    face.outline_glyph(id, &mut builder)?;
    if builder.count > limits.segments {
        return None;
    }
    builder.close_contour();
    Some(builder.shape)
}

struct Builder {
    shape:  Shape<Contour>,
    start:  Option<Point>,
    last:   Option<Point>,
    count:  usize,
    limits: Limits,
}

impl Builder {
    fn push(&mut self, segment: Segment, end: Point) {
        self.count += 1;
        if self.count > self.limits.segments {
            return;
        }
        if let Some(contour) = self.shape.contours.last_mut() {
            contour.segments.push(segment);
            self.last = Some(end);
        }
    }

    /// Closes the open contour with a straight run back to its start, which is
    /// what an outline means whether or not the face says `close`.
    fn close_contour(&mut self) {
        let (Some(start), Some(last)) = (self.start, self.last) else {
            return;
        };
        if start != last {
            self.push(Segment::line(last, start), start);
        }
    }
}

impl OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close_contour();
        let point = Point::new(x.into(), y.into());
        self.start = Some(point);
        self.last = Some(point);
        self.shape.contours.push(Contour::default());
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let Some(last) = self.last else { return };
        let point = Point::new(x.into(), y.into());
        self.push(Segment::line(last, point), point);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let Some(last) = self.last else { return };
        let point = Point::new(x.into(), y.into());
        self.push(
            Segment::quad(last, Point::new(x1.into(), y1.into()), point),
            point,
        );
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let Some(last) = self.last else { return };
        let point = Point::new(x.into(), y.into());
        self.push(
            Segment::cubic(
                last,
                Point::new(x1.into(), y1.into()),
                Point::new(x2.into(), y2.into()),
                point,
            ),
            point,
        );
    }

    fn close(&mut self) {
        self.close_contour();
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

    fn segments(shape: &Shape<Contour>) -> usize {
        shape
            .contours
            .iter()
            .map(|contour| contour.segments.len())
            .sum()
    }

    #[test]
    fn a_letter_loads_as_closed_contours() {
        let font = font();
        let id = font.glyph_index('O').expect("O");
        let shape = load(font.face(), id, Limits::default()).expect("outline");
        assert_eq!(shape.contours.len(), 2, "an O is a ring inside a ring");
        for contour in &shape.contours {
            let first = contour.segments.first().expect("segment").start();
            let last = contour.segments.last().expect("segment").end();
            assert!((first - last).norm() < 1.0e-6, "the contour closes");
        }
    }

    #[test]
    fn a_space_has_no_outline() {
        let font = font();
        let id = font.glyph_index(' ').expect("space");
        assert!(load(font.face(), id, Limits::default()).is_none());
    }

    #[test]
    fn an_outline_past_the_cap_is_refused_whole() {
        let font = font();
        let id = font.glyph_index('@').expect("@");
        assert!(
            load(font.face(), id, Limits { segments: 4 }).is_none(),
            "a glyph whose field would take minutes never starts"
        );
        assert!(
            segments(&load(font.face(), id, Limits::default()).expect("outline")) > 4,
            "and the same glyph loads under the real cap"
        );
    }

    #[test]
    fn a_segment_before_any_contour_is_dropped_rather_than_panicking() {
        let mut builder = Builder {
            shape:  Shape::default(),
            start:  None,
            last:   None,
            count:  0,
            limits: Limits::default(),
        };
        builder.line_to(1.0, 1.0);
        builder.quad_to(1.0, 1.0, 2.0, 2.0);
        builder.curve_to(1.0, 1.0, 2.0, 2.0, 3.0, 3.0);
        builder.close();
        assert!(builder.shape.contours.is_empty());
    }
}
