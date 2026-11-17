//! A pane: a framed page of blocks, laid out in ems and lines with no host
//! round trip.
//!
//! Every length here is in ems of the body type, measured against Noto Sans,
//! and every height is in lines. `x` runs right and `y` down from the pane's
//! top left. The binding multiplies by an em in metres and does nothing else.

use std::collections::HashMap;

use smol_str::SmolStr;
use wired_guest::{
    math::Vec2,
    style,
};

use crate::component::pane::{
    block::{
        Block,
        Kind,
    },
    control::Control,
    layout::Laid,
};

pub mod block;
pub mod control;
pub mod draw;
pub mod focus;
pub mod hit;
pub mod layout;
pub mod panes;
pub mod pool;
pub mod prims;
pub mod scroll;

#[cfg(test)] mod tests;

/// Ems across one column.
pub const COLUMN: f32 = style::MONO_ADVANCE;
/// Ems from one line to the next.
pub const LINE: f32 = style::MONO_LINE;
/// Ems between the pane's edge and its content, sideways and vertically.
pub const PAD_X: f32 = style::MONO_PAD_COLUMNS * COLUMN;
pub const PAD_Y: f32 = style::MONO_PAD_LINES * LINE;
/// The title's and the status line's size, as a multiple of the body's.
pub const TITLE_SCALE: f32 = style::MRAD_TITLE / style::MRAD_BODY;
pub const SMALL_SCALE: f32 = style::MRAD_SMALL / style::MRAD_BODY;

/// Widths a pane is made at, in columns.
pub const WIDTHS: [u32; 4] = style::MONO_COLUMNS;
/// Most body lines a pane has.
pub const MAX_LINES: u32 = 40;
pub const MAX_BLOCKS: usize = 8;
/// Most rows a block holds.
pub const MAX_ROWS: usize = 512;
/// Longest string a pane holds, in bytes.
pub const MAX_TEXT_BYTES: usize = 1024;

/// Where the baseline sits in a line box, in ems of the text's size.
pub const BASELINE: f32 = 1.1;

/// Why a script's pane was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    Width(u32),
    Lines(u32),
    Blocks(usize),
    Rows(usize),
    Text(usize),
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Width(columns) => write!(f, "{columns} columns, not one of {WIDTHS:?}"),
            Self::Lines(lines) => write!(f, "{lines} lines, not from 1 to {MAX_LINES}"),
            Self::Blocks(count) => write!(f, "{count} blocks, over the limit of {MAX_BLOCKS}"),
            Self::Rows(count) => {
                write!(f, "a block has {count} rows, over the limit of {MAX_ROWS}")
            }
            Self::Text(len) => {
                write!(
                    f,
                    "a string is {len} bytes, over the limit of {MAX_TEXT_BYTES}"
                )
            }
        }
    }
}

/// What a pane did, in the terms a consumer thinks in. Each names the id the
/// consumer gave.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Pressed(u32),
    /// Asks for this state; the pane draws what the consumer sets.
    Toggled(u32, bool),
    /// A slider was pressed or dragged to this fraction.
    Slid(u32, f32),
    /// A number was dragged by this much of its value since the last report.
    Scrubbed(u32, f32),
    /// A drag on a slider or number ended.
    Released(u32),
    Chosen(u32, u32),
    Selected(u32),
    Folded(u32, bool),
    Followed(u32),
}

/// A rectangle in ems.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    #[must_use]
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x < self.x + self.w
            && point.y >= self.y
            && point.y < self.y + self.h
    }
}

/// The frame around the body, in ems.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub width:      f32,
    pub height:     f32,
    pub body_top:   f32,
    pub status_top: Option<f32>,
}

/// What a pane keeps for the viewer's sake: where each block is scrolled,
/// what the pointer is over, and what has the keyboard. Never synced.
#[derive(Debug, Default, Clone)]
pub struct View {
    scroll:  HashMap<u32, u32>,
    hovered: Option<u32>,
    pressed: Option<u32>,
    focus:   Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Pane {
    columns: u32,
    /// Body lines, or none to grow to fit up to [`MAX_LINES`].
    lines:   Option<u32>,
    title:   SmolStr,
    status:  Option<SmolStr>,
    blocks:  Vec<Block>,
    laid:    Laid,
    view:    View,
}

impl Pane {
    pub fn new(columns: u32, lines: Option<u32>) -> Result<Self, Refused> {
        if !WIDTHS.contains(&columns) {
            return Err(Refused::Width(columns));
        }
        if let Some(lines) = lines.filter(|lines| !(1..=MAX_LINES).contains(lines)) {
            return Err(Refused::Lines(lines));
        }
        let mut pane = Self {
            columns,
            lines,
            title: SmolStr::default(),
            status: None,
            blocks: Vec::new(),
            laid: Laid::default(),
            view: View::default(),
        };
        pane.relay();
        Ok(pane)
    }

    pub fn set_title(&mut self, title: &str) -> Result<(), Refused> {
        check_text(title)?;
        self.title = SmolStr::new(title);
        Ok(())
    }

    pub fn set_status(&mut self, status: Option<&str>) -> Result<(), Refused> {
        if let Some(status) = status {
            check_text(status)?;
        }
        self.status = status.map(SmolStr::new);
        Ok(())
    }

    /// Replaces the blocks and returns whether they differ from what was
    /// there. A block keeps its scroll by its id.
    pub fn set_blocks(&mut self, blocks: Vec<Block>) -> Result<bool, Refused> {
        if blocks.len() > MAX_BLOCKS {
            return Err(Refused::Blocks(blocks.len()));
        }
        for block in &blocks {
            check_block(&block.kind)?;
        }
        if blocks == self.blocks {
            return Ok(false);
        }
        self.blocks = blocks;
        self.relay();
        Ok(true)
    }

    fn relay(&mut self) {
        self.laid = layout::lay_out(&self.blocks, self.width(), self.lines);
        let ids = self.blocks.iter().map(|block| block.id).collect::<Vec<_>>();
        self.view.scroll.retain(|id, _| ids.contains(id));
        for block in &self.laid.blocks {
            if let Some(offset) = self.view.scroll.get_mut(&block.id) {
                *offset = scroll::clamp(*offset, block.content(), block.window());
            }
        }
    }

    /// Ems across the whole pane.
    #[must_use]
    pub fn width(&self) -> f32 {
        self.columns as f32 * COLUMN
    }

    #[must_use]
    pub const fn laid(&self) -> &Laid {
        &self.laid
    }

    #[must_use]
    pub const fn view(&self) -> &View {
        &self.view
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    #[must_use]
    pub fn frame(&self) -> Frame {
        let title = if self.title.is_empty() {
            0.0
        } else {
            LINE.mul_add(TITLE_SCALE, PAD_Y)
        };
        let body_top = PAD_Y + title;
        let body_end = LINE.mul_add(self.laid.lines as f32, body_top);
        let (status_top, bottom) = if self.status.is_some() {
            (
                Some(body_end + PAD_Y),
                LINE.mul_add(SMALL_SCALE, body_end + PAD_Y),
            )
        } else {
            (None, body_end)
        };
        Frame {
            width: self.width(),
            height: bottom + PAD_Y,
            body_top,
            status_top,
        }
    }

    /// Width and height in ems.
    #[must_use]
    pub fn size(&self) -> Vec2 {
        let frame = self.frame();
        Vec2::new(frame.width, frame.height)
    }

    #[must_use]
    pub fn scroll_of(&self, block: u32) -> u32 {
        self.view.scroll.get(&block).copied().unwrap_or(0)
    }

    /// Scrolls a block by `rows`, and returns whether it moved.
    pub fn scroll_by(&mut self, block: u32, rows: i32) -> bool {
        let Some(laid) = self.laid.blocks.iter().find(|laid| laid.id == block) else {
            return false;
        };
        let now = self.scroll_of(block);
        let next = scroll::by(now, rows, laid.content(), laid.window());
        self.view.scroll.insert(block, next);
        next != now
    }

    /// Scrolls a block to where a press at `fraction` of its rail asks.
    pub fn scroll_to(&mut self, block: u32, fraction: f32) -> bool {
        let Some(laid) = self.laid.blocks.iter().find(|laid| laid.id == block) else {
            return false;
        };
        let now = self.scroll_of(block);
        let next = scroll::at(fraction, laid.content(), laid.window());
        self.view.scroll.insert(block, next);
        next != now
    }

    /// Returns whether what the pointer is over changed.
    pub fn set_hovered(&mut self, id: Option<u32>) -> bool {
        std::mem::replace(&mut self.view.hovered, id) != id
    }

    pub fn set_pressed(&mut self, id: Option<u32>) -> bool {
        std::mem::replace(&mut self.view.pressed, id) != id
    }

    pub fn set_focus(&mut self, id: Option<u32>) -> bool {
        std::mem::replace(&mut self.view.focus, id) != id
    }

    #[must_use]
    pub const fn focus(&self) -> Option<u32> {
        self.view.focus
    }
}

fn check_text(text: &str) -> Result<(), Refused> {
    if text.len() > MAX_TEXT_BYTES {
        return Err(Refused::Text(text.len()));
    }
    Ok(())
}

fn check_control(control: &Control) -> Result<(), Refused> {
    match control {
        Control::Value(text) | Control::Link(text) | Control::Action { label: text, .. } => {
            check_text(text)
        }
        Control::Choice { options, .. } => options.iter().try_for_each(|option| check_text(option)),
        _ => Ok(()),
    }
}

fn check_block(kind: &Kind) -> Result<(), Refused> {
    let rows = match kind {
        Kind::Text(spans) => {
            for span in spans {
                check_text(&span.text)?;
            }
            spans.len()
        }
        Kind::Form(rows) => {
            for row in rows {
                check_text(&row.label)?;
                check_control(&row.control)?;
            }
            rows.len()
        }
        Kind::Table(table) => {
            for row in &table.rows {
                for cell in &row.cells {
                    check_text(cell)?;
                }
            }
            table.rows.len()
        }
        Kind::Tree(nodes) => {
            for node in nodes {
                check_text(&node.label)?;
            }
            nodes.len()
        }
        Kind::Bar(items) => {
            for item in items {
                check_control(&item.control)?;
            }
            items.len()
        }
    };
    if rows > MAX_ROWS {
        return Err(Refused::Rows(rows));
    }
    Ok(())
}
