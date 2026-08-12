use crate::atlas::{
    GlyphSource,
    Rect,
};

/// Cap on the glyphs one string may lay out; past it the pipeline errors
/// rather than running long.
pub const MAX_GLYPHS: usize = 4096;

/// Where the origin sits on a line, against the line's own width rather than
/// the wrap box.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutOpts {
    /// Em size, in the same units the caller wants the result in.
    pub size:        f32,
    /// Wrap width. `None` breaks only on newlines.
    pub wrap:        Option<f32>,
    pub align:       Align,
    /// Multiple of the font's own baseline-to-baseline distance.
    pub line_height: f32,
    pub max_glyphs:  usize,
}

impl Default for LayoutOpts {
    fn default() -> Self {
        Self {
            size:        0.02,
            wrap:        None,
            align:       Align::Left,
            line_height: 1.0,
            max_glyphs:  MAX_GLYPHS,
        }
    }
}

/// One glyph to draw. `uv.min` is the top-left texel of `plane`'s top-left
/// corner; the plane is y-up and the atlas is y-down. `page` names the atlas
/// page the uv is relative to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub plane: Rect,
    pub uv:    Rect,
    pub page:  u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Laid {
    pub quads:   Vec<Quad>,
    /// The metric box a backing surface should be sized to.
    pub bounds:  Rect,
    pub ink:     Rect,
    pub lines:   usize,
    /// Characters the source fell back on rather than drawing a real glyph.
    /// The caller decides what to do with them; a runtime atlas requests them.
    pub missing: Vec<char>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LayoutError {
    #[error("{count} glyphs exceeds the {cap} cap")]
    TooManyGlyphs { count: usize, cap: usize },
}

/// A glyph at `pen` em units from the line's start.
#[derive(Debug, Clone, Copy)]
struct Placed {
    ch:  char,
    pen: f32,
}

#[derive(Debug, Default)]
struct Line {
    placed: Vec<Placed>,
    pen:    f32,
}

impl Line {
    fn last(&self) -> Option<char> {
        self.placed.last().map(|placed| placed.ch)
    }

    /// Ink width, ignoring trailing spaces so a wrapped centred line does not
    /// drift by the spaces at each break.
    fn width(&self, source: &impl GlyphSource) -> f32 {
        self.placed
            .iter()
            .rev()
            .find(|placed| !placed.ch.is_whitespace())
            .map_or(0.0, |placed| {
                placed.pen + source.glyph(placed.ch).map_or(0.0, |glyph| glyph.advance)
            })
    }
}

pub fn layout(
    text: &str,
    source: &impl GlyphSource,
    opts: &LayoutOpts,
) -> Result<Laid, LayoutError> {
    let count = text.chars().count();
    if count > opts.max_glyphs {
        return Err(LayoutError::TooManyGlyphs {
            count,
            cap: opts.max_glyphs,
        });
    }

    let wrap_em = opts
        .wrap
        .filter(|wrap| *wrap > 0.0 && opts.size > 0.0)
        .map(|wrap| wrap / opts.size);

    let mut lines = Vec::new();
    let mut line = Line::default();
    let mut missing = Vec::new();

    for ch in text.chars() {
        if ch == '\n' {
            lines.push(std::mem::take(&mut line));
            continue;
        }
        if source.missing(ch) && !missing.contains(&ch) {
            missing.push(ch);
        }
        let Some(glyph) = source.glyph(ch) else {
            continue;
        };

        let pen = line.pen + line.last().map_or(0.0, |prev| source.kern(prev, ch));
        if let Some(wrap) = wrap_em
            && !line.placed.is_empty()
            && pen + glyph.advance > wrap
        {
            line = wrap_line(&mut lines, line, source);
        }

        let pen = line.pen + line.last().map_or(0.0, |prev| source.kern(prev, ch));
        line.placed.push(Placed { ch, pen });
        line.pen = pen + glyph.advance;
    }
    lines.push(line);

    Ok(assemble(&lines, source, opts, missing))
}

/// Splits a word wider than the wrap box mid-word rather than letting it
/// overflow the box the caller asked for.
fn wrap_line(lines: &mut Vec<Line>, line: Line, source: &impl GlyphSource) -> Line {
    let break_at = line
        .placed
        .iter()
        .rposition(|placed| placed.ch.is_whitespace())
        .map_or(line.placed.len(), |space| space + 1);

    let mut head = line;
    let tail = head.placed.split_off(break_at);
    head.pen = head.width(source);
    lines.push(head);

    let mut next = Line::default();
    let origin = tail.first().map_or(0.0, |placed| placed.pen);
    for placed in tail {
        let pen = placed.pen - origin;
        next.pen = pen + source.glyph(placed.ch).map_or(0.0, |glyph| glyph.advance);
        next.placed.push(Placed { pen, ..placed });
    }
    next
}

fn assemble(
    lines: &[Line],
    source: &impl GlyphSource,
    opts: &LayoutOpts,
    missing: Vec<char>,
) -> Laid {
    let step = source.vertical().line_height() * opts.line_height;
    let mut quads = Vec::new();
    let mut ink = None;
    let mut bounds = None;

    for (index, line) in lines.iter().enumerate() {
        let width = line.width(source);
        let offset = match opts.align {
            Align::Left => 0.0,
            Align::Center => -width / 2.0,
            Align::Right => -width,
        };
        let baseline = -(index as f32) * step;

        let metric = Rect {
            min: [offset, baseline + source.vertical().descender],
            max: [offset + width, baseline + source.vertical().ascender],
        }
        .scaled(opts.size);
        merge(&mut bounds, metric);

        for placed in &line.placed {
            let Some(glyph) = source.glyph(placed.ch) else {
                continue;
            };
            if glyph.plane.is_empty() {
                continue;
            }
            let plane = glyph
                .plane
                .translated(placed.pen + offset, baseline)
                .scaled(opts.size);
            merge(&mut ink, plane);
            quads.push(Quad {
                plane,
                uv: glyph.uv,
                page: glyph.page,
            });
        }
    }

    Laid {
        quads,
        bounds: bounds.unwrap_or(Rect::ZERO),
        ink: ink.unwrap_or(Rect::ZERO),
        lines: lines.len(),
        missing,
    }
}

const fn merge(target: &mut Option<Rect>, rect: Rect) {
    match target {
        Some(current) => current.union(&rect),
        None => *target = Some(rect),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::atlas::{
        Atlas,
        Glyph,
        VerticalMetrics,
    };

    /// Every glyph one em wide and half an em tall, so a width is a character
    /// count.
    fn atlas() -> Atlas {
        let square = Glyph {
            plane:   Rect {
                min: [0.0, 0.0],
                max: [1.0, 0.5],
            },
            uv:      Rect {
                min: [0.0, 0.0],
                max: [0.1, 0.1],
            },
            advance: 1.0,
            page:    0,
        };
        let blank = Glyph {
            plane: Rect::ZERO,
            uv: Rect::ZERO,
            ..square
        };

        let mut glyphs = BTreeMap::new();
        for ch in 'a'..='z' {
            glyphs.insert(ch, square);
        }
        for ch in ['A', 'V'] {
            glyphs.insert(ch, square);
        }
        glyphs.insert(' ', blank);

        Atlas {
            width: 128,
            height: 128,
            range: 4.0,
            vertical: VerticalMetrics {
                ascender:  0.75,
                descender: -0.25,
                line_gap:  0.0,
            },
            glyphs,
            kerning: BTreeMap::from([(('A', 'V'), -0.5)]),
        }
    }

    fn opts() -> LayoutOpts {
        LayoutOpts {
            size: 1.0,
            ..Default::default()
        }
    }

    fn laid(text: &str, opts: &LayoutOpts) -> Laid {
        layout(text, &atlas(), opts).expect("layout")
    }

    /// Line starts, top line first.
    fn line_starts(laid: &Laid) -> Vec<f32> {
        let mut starts = Vec::new();
        let mut current: Option<(f32, f32)> = None;
        for quad in &laid.quads {
            match current {
                Some((y, _)) if (y - quad.plane.min[1]).abs() < 1.0e-4 => {}
                _ => {
                    starts.push(quad.plane.min[0]);
                    current = Some((quad.plane.min[1], quad.plane.min[0]));
                }
            }
        }
        starts
    }

    #[test]
    fn glyphs_advance_along_the_baseline() {
        let laid = laid("abc", &opts());
        assert_eq!(laid.quads.len(), 3);
        assert!((laid.quads[0].plane.min[0]).abs() < 1.0e-4);
        assert!((laid.quads[1].plane.min[0] - 1.0).abs() < 1.0e-4);
        assert!((laid.quads[2].plane.min[0] - 2.0).abs() < 1.0e-4);
    }

    #[test]
    fn size_scales_the_result() {
        let big = laid(
            "abc",
            &LayoutOpts {
                size: 2.0,
                ..opts()
            },
        );
        assert!((big.quads[2].plane.min[0] - 4.0).abs() < 1.0e-4);
    }

    #[test]
    fn a_kerned_pair_pulls_together() {
        let laid = laid("AV", &opts());
        assert!(
            (laid.quads[1].plane.min[0] - 0.5).abs() < 1.0e-4,
            "the pair's -0.5 em adjustment applies once"
        );
    }

    #[test]
    fn a_newline_starts_a_line_below() {
        let laid = laid("ab\ncd", &opts());
        assert_eq!(laid.lines, 2);
        assert!(laid.quads[2].plane.min[1] < laid.quads[0].plane.min[1]);
        assert!((line_starts(&laid)[1]).abs() < 1.0e-4);
    }

    #[test]
    fn line_height_spaces_the_baselines() {
        let single = laid("a\nb", &opts());
        let double = laid(
            "a\nb",
            &LayoutOpts {
                line_height: 2.0,
                ..opts()
            },
        );
        let drop = |laid: &Laid| laid.quads[0].plane.min[1] - laid.quads[1].plane.min[1];
        assert!(2.0f32.mul_add(-drop(&single), drop(&double)).abs() < 1.0e-4);
    }

    #[test]
    fn wrapping_breaks_between_words() {
        let laid = laid(
            "aa bb cc",
            &LayoutOpts {
                wrap: Some(5.5),
                ..opts()
            },
        );
        assert_eq!(laid.lines, 2);
        assert!(
            (line_starts(&laid)[1]).abs() < 1.0e-4,
            "the wrapped word starts the line, not the space before it"
        );
    }

    #[test]
    fn a_word_wider_than_the_box_breaks_rather_than_overflowing() {
        let wrap = 3.0;
        let laid = laid(
            "abcdefgh",
            &LayoutOpts {
                wrap: Some(wrap),
                ..opts()
            },
        );
        assert!(laid.lines > 1);
        assert!(
            laid.ink.max[0] <= wrap + 1.0e-4,
            "nothing escapes the box the caller asked for"
        );
    }

    #[test]
    fn trailing_spaces_do_not_widen_a_line() {
        let bare = laid("ab", &opts());
        let trailing = laid("ab   ", &opts());
        assert!((bare.bounds.max[0] - trailing.bounds.max[0]).abs() < 1.0e-4);
    }

    #[test]
    fn centring_puts_the_origin_in_the_middle() {
        let laid = laid(
            "abcd",
            &LayoutOpts {
                align: Align::Center,
                ..opts()
            },
        );
        assert!((laid.bounds.min[0] + laid.bounds.max[0]).abs() < 1.0e-4);
    }

    #[test]
    fn right_alignment_ends_at_the_origin() {
        let laid = laid(
            "abcd",
            &LayoutOpts {
                align: Align::Right,
                ..opts()
            },
        );
        assert!((laid.bounds.max[0]).abs() < 1.0e-4);
        assert!((laid.bounds.min[0] + 4.0).abs() < 1.0e-4);
    }

    #[test]
    fn each_wrapped_line_is_centred_on_its_own_width() {
        let laid = laid(
            "aa bbbb",
            &LayoutOpts {
                align: Align::Center,
                wrap: Some(4.5),
                ..opts()
            },
        );
        assert_eq!(laid.lines, 2);
        let starts = line_starts(&laid);
        assert!((starts[0] + 1.0).abs() < 1.0e-4);
        assert!((starts[1] + 2.0).abs() < 1.0e-4);
    }

    #[test]
    fn a_blank_glyph_advances_without_drawing() {
        let laid = laid("a b", &opts());
        assert_eq!(laid.quads.len(), 2, "the space leaves no ink");
        assert!((laid.quads[1].plane.min[0] - 2.0).abs() < 1.0e-4);
    }

    #[test]
    fn a_character_the_source_lacks_is_reported_not_swallowed() {
        let laid = laid("a漢b漢", &opts());
        assert_eq!(laid.missing, vec!['漢']);
        assert_eq!(laid.quads.len(), 2);
    }

    #[test]
    fn the_metric_box_spans_ascender_to_descender() {
        let laid = laid("a", &opts());
        assert!((laid.bounds.max[1] - 0.75).abs() < 1.0e-4);
        assert!((laid.bounds.min[1] + 0.25).abs() < 1.0e-4);
        assert!(
            laid.ink.max[1] < laid.bounds.max[1],
            "ink is not the box a backing surface wants"
        );
    }

    #[test]
    fn an_empty_string_lays_out_to_nothing() {
        let laid = laid("", &opts());
        assert!(laid.quads.is_empty());
        assert_eq!(laid.ink, Rect::ZERO);
        assert_eq!(laid.lines, 1);
    }

    #[test]
    fn too_much_text_is_an_error_rather_than_a_slow_frame() {
        let opts = LayoutOpts {
            max_glyphs: 4,
            ..opts()
        };
        assert_eq!(
            layout("abcde", &atlas(), &opts),
            Err(LayoutError::TooManyGlyphs { count: 5, cap: 4 })
        );
    }

    #[test]
    fn a_degenerate_wrap_width_does_not_hang() {
        let laid = laid(
            "abc",
            &LayoutOpts {
                wrap: Some(0.0),
                ..opts()
            },
        );
        assert_eq!(laid.quads.len(), 3, "a zero box wraps nothing");
    }
}
