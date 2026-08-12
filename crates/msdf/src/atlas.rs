/// An axis-aligned box; a glyph is a pair, one in em units and one in texture
/// coordinates.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Glyph {
    pub plane:   Rect,
    pub uv:      Rect,
    pub advance: f32,
    /// Which page of a multi-page atlas the field lives in.
    pub page:    u32,
    /// Which font in a fallback stack the glyph came from; a single-font
    /// source always stamps 0.
    pub font:    u32,
}

/// Line spacing in em units.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
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

/// What laying text out needs from a glyph store: vertical metrics, per-char
/// placement, and pair adjustments. A [`crate::runtime::Atlas`] and a fallback
/// stack over several both satisfy it.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_height_spans_ascender_to_descender_plus_the_gap() {
        let vertical = VerticalMetrics {
            ascender:  0.8,
            descender: -0.2,
            line_gap:  0.1,
        };
        assert!((vertical.line_height() - 1.1).abs() < 1.0e-6);
    }

    #[test]
    fn a_zero_area_rect_is_empty() {
        assert!(Rect::ZERO.is_empty());
        assert!(
            !Rect {
                min: [0.0, 0.0],
                max: [0.5, 0.5],
            }
            .is_empty()
        );
    }

    #[test]
    fn a_union_covers_both_rects() {
        let mut rect = Rect {
            min: [0.0, 0.0],
            max: [1.0, 1.0],
        };
        let other = Rect {
            min: [-2.0, 0.5],
            max: [0.5, 3.0],
        };
        rect.union(&other);

        for corner in [other.min, other.max, [0.0, 0.0], [1.0, 1.0]] {
            assert!(
                (rect.min[0]..=rect.max[0]).contains(&corner[0])
                    && (rect.min[1]..=rect.max[1]).contains(&corner[1]),
                "{corner:?} falls outside {rect:?}"
            );
        }
    }
}
