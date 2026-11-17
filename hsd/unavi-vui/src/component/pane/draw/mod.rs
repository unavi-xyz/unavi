//! A pane as a list of runs: text and filled rectangles in ems, which the
//! binding transcribes into prims.

use smol_str::SmolStr;
use wired_guest::math::Vec2;

use crate::{
    component::pane::{
        BASELINE,
        COLUMN,
        LINE,
        PAD_X,
        PAD_Y,
        Pane,
        Rect,
        SMALL_SCALE,
        TITLE_SCALE,
        block::Tone,
        layout::{
            Item,
            LaidBlock,
            LaidRow,
        },
        scroll,
    },
    face::measure::text,
};

mod control;

/// Ems thick a hairline is.
const HAIRLINE: f32 = 0.08;
/// Ems a link's underline sits below its baseline.
const UNDERLINE_DROP: f32 = 0.14;

/// What a rectangle is filled as. The binding chooses the colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    /// Under what the pointer is over, and behind an action.
    Tint,
    Rule,
    /// The unlit part of a slider, meter or toggle.
    Track,
    /// The lit part of a slider, meter or toggle.
    Lit,
    /// Under what is pressed, selected or primary.
    Accent,
    /// The bead on a rail.
    Thumb,
    Underline,
    /// A line under what has the keyboard.
    Focus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextRun {
    pub text:   SmolStr,
    /// Left edge and baseline.
    pub at:     Vec2,
    /// A multiple of the body type's size.
    pub scale:  f32,
    pub tone:   Tone,
    pub width:  f32,
    /// The id a press on this run reports.
    pub target: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RectRun {
    pub rect:   Rect,
    pub fill:   Fill,
    /// The id a press on this run reports.
    pub target: Option<u32>,
}

/// Everything a pane draws, back to front within each list.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Runs {
    pub text:  Vec<TextRun>,
    pub rects: Vec<RectRun>,
    /// What the pointer and keyboard put on the pane, drawn after every
    /// other rectangle so a change in them does not shift the rest.
    marks:     Vec<RectRun>,
}

impl Runs {
    fn rect(&mut self, rect: Rect, fill: Fill, target: Option<u32>) {
        self.rects.push(RectRun { rect, fill, target });
    }

    fn mark(&mut self, rect: Rect, fill: Fill, target: Option<u32>) {
        self.marks.push(RectRun { rect, fill, target });
    }

    fn text(&mut self, text: &str, at: Vec2, scale: f32, tone: Tone, target: Option<u32>) {
        if text.is_empty() {
            return;
        }
        self.text.push(TextRun {
            text: SmolStr::new(text),
            at,
            scale,
            tone,
            width: text::width(text) * scale,
            target,
        });
    }
}

impl Pane {
    /// Every run of the pane as it stands: frame, blocks, and the state the
    /// pointer and keyboard put on them.
    #[must_use]
    pub fn runs(&self) -> Runs {
        let mut runs = Runs::default();
        let frame = self.frame();
        let inner = frame.width - 2.0 * PAD_X;

        if !self.title().is_empty() {
            let title = text::truncate(self.title(), inner / TITLE_SCALE);
            runs.text(
                &title,
                Vec2::new(PAD_X, BASELINE.mul_add(TITLE_SCALE, PAD_Y)),
                TITLE_SCALE,
                Tone::Body,
                None,
            );
            runs.rect(
                Rect {
                    x: PAD_X,
                    y: frame.body_top - PAD_Y / 2.0 - HAIRLINE / 2.0,
                    w: inner,
                    h: HAIRLINE,
                },
                Fill::Rule,
                None,
            );
        }
        if let (Some(status), Some(top)) = (self.status(), frame.status_top) {
            let status = text::truncate(status, inner / SMALL_SCALE);
            runs.text(
                &status,
                Vec2::new(PAD_X, BASELINE.mul_add(SMALL_SCALE, top)),
                SMALL_SCALE,
                Tone::Muted,
                None,
            );
        }

        for block in &self.laid().blocks {
            self.draw_block(&mut runs, block);
        }
        let marks = std::mem::take(&mut runs.marks);
        runs.rects.extend(marks);
        runs
    }

    fn draw_block(&self, runs: &mut Runs, block: &LaidBlock) {
        let frame = self.frame();
        let offset = self.scroll_of(block.id);
        let top = LINE.mul_add(block.top as f32, frame.body_top);
        let rail = if block.rail { COLUMN } else { 0.0 };
        let inner = frame.width - 2.0 * PAD_X - rail;

        for (line, row) in block.visible(offset) {
            let y = LINE.mul_add(line as f32, top);
            self.draw_row(runs, row, y, inner);
        }

        if block.rail {
            let height = LINE * block.lines as f32;
            let x = frame.width - PAD_X - COLUMN;
            runs.rect(
                Rect {
                    x,
                    y: top,
                    w: COLUMN,
                    h: height,
                },
                Fill::Track,
                None,
            );
            let thumb = scroll::thumb(offset, block.content(), block.window());
            runs.rect(
                Rect {
                    x,
                    y: thumb.start.mul_add(height, top),
                    w: COLUMN,
                    h: thumb.len * height,
                },
                Fill::Thumb,
                None,
            );
        }
    }

    fn draw_row(&self, runs: &mut Runs, row: &LaidRow, y: f32, inner: f32) {
        let view = self.view();
        if let Some(select) = row.select {
            let whole = Rect {
                x: PAD_X,
                y,
                w: inner,
                h: LINE,
            };
            if select.selected {
                runs.mark(whole, Fill::Accent, Some(select.id));
            } else if view.hovered == Some(select.id) {
                runs.mark(whole, Fill::Tint, Some(select.id));
            }
        }
        for item in &row.items {
            match item {
                Item::Text {
                    text,
                    x,
                    width,
                    tone,
                    link,
                } => {
                    let baseline = Vec2::new(*x, BASELINE + y);
                    runs.text(text, baseline, 1.0, *tone, *link);
                    if link.is_some() {
                        runs.rect(
                            Rect {
                                x:  *x,
                                y:  baseline.y + UNDERLINE_DROP,
                                w:  *width,
                                h:  HAIRLINE,
                            },
                            Fill::Underline,
                            *link,
                        );
                    }
                }
                Item::Fold { id, x, open, .. } => {
                    let mark = if *open { "\u{2212}" } else { "+" };
                    runs.text(mark, Vec2::new(*x, BASELINE + y), 1.0, Tone::Muted, Some(*id));
                }
                Item::Control {
                    id,
                    x,
                    width,
                    control,
                } => self.draw_control(runs, *id, control, Rect {
                    x:  *x,
                    y,
                    w:  *width,
                    h:  LINE,
                }),
            }
        }
    }
}
