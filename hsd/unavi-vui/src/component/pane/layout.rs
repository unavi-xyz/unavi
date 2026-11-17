//! Blocks to rows: what each block shows, where it sits in the body, and
//! how much of it there is.

use smol_str::SmolStr;

use crate::{
    component::pane::{
        COLUMN,
        MAX_LINES,
        PAD_X,
        block::{
            Block,
            Height,
            Kind,
            Node,
            Span,
            Table,
            Tone,
        },
        control::Control,
    },
    face::measure::text,
};

/// Share of a form's width the labels take.
const LABEL_SHARE: f32 = 0.4;
/// Ems a tree row steps in for each level of depth.
const INDENT: f32 = 2.0 * COLUMN;
/// Ems a tree row reserves for its fold mark.
const FOLD_EM: f32 = 2.0 * COLUMN;

/// The pane's body once laid out.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Laid {
    pub blocks: Vec<LaidBlock>,
    /// Body lines, which blocks stack down.
    pub lines:  u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidBlock {
    pub id:     u32,
    /// Lines from the top of the body.
    pub top:    u32,
    pub lines:  u32,
    /// Rows at the front that stay put while the rest scroll: a table's
    /// header.
    pub pinned: usize,
    pub rows:   Vec<LaidRow>,
    /// Whether a rail takes the block's last column.
    pub rail:   bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LaidRow {
    pub items:  Vec<Item>,
    pub select: Option<Select>,
}

/// A row a press selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Select {
    pub id:       u32,
    pub selected: bool,
}

/// One thing on a row. `x` is ems from the pane's left edge.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Text {
        text:  SmolStr,
        x:     f32,
        width: f32,
        tone:  Tone,
        link:  Option<u32>,
    },
    Control {
        id:      u32,
        x:       f32,
        width:   f32,
        control: Control,
    },
    Fold {
        id:    u32,
        x:     f32,
        width: f32,
        open:  bool,
    },
}

impl LaidRow {
    const fn of(items: Vec<Item>) -> Self {
        Self {
            items,
            select: None,
        }
    }
}

impl LaidBlock {
    /// Rows that scroll.
    #[must_use]
    pub const fn content(&self) -> usize {
        self.rows.len().saturating_sub(self.pinned)
    }

    /// Lines the scrolling rows show through.
    #[must_use]
    pub const fn window(&self) -> u32 {
        self.lines.saturating_sub(self.pinned as u32)
    }

    /// The rows on view at `offset`, each with the line of the block it is
    /// drawn on.
    pub fn visible(&self, offset: u32) -> impl Iterator<Item = (u32, &LaidRow)> {
        let pinned = self.pinned.min(self.rows.len());
        let shown = self.rows[..pinned].iter().enumerate();
        let scrolled = self.rows[pinned..]
            .iter()
            .skip(offset as usize)
            .take(self.window() as usize)
            .enumerate();
        shown
            .map(|(line, row)| (line as u32, row))
            .chain(scrolled.map(move |(line, row)| (self.pinned as u32 + line as u32, row)))
            .take(self.lines as usize)
    }

    /// The row drawn on `line` of the block.
    #[must_use]
    pub fn row_at(&self, line: u32, offset: u32) -> Option<&LaidRow> {
        self.visible(offset)
            .find(|(drawn, _)| *drawn == line)
            .map(|(_, row)| row)
    }
}

/// Lays `blocks` out across `width` ems and `lines` body lines, or as many
/// lines as they need.
#[must_use]
pub fn lay_out(blocks: &[Block], width: f32, lines: Option<u32>) -> Laid {
    let content = width - 2.0 * PAD_X;
    let mut rows = blocks
        .iter()
        .map(|block| block_rows(block, content))
        .collect::<Vec<_>>();

    let lines = lines.unwrap_or_else(|| {
        let wanted = blocks
            .iter()
            .zip(&rows)
            .map(|(block, (rows, _))| match block.height {
                Height::Lines(lines) => lines as usize,
                Height::Share => rows.len(),
            });
        wanted.sum::<usize>().clamp(1, MAX_LINES as usize) as u32
    });

    let heights = heights(blocks, lines);
    let mut placed = Vec::with_capacity(blocks.len());
    let mut top = 0;
    for ((block, height), (block_rows, pinned)) in blocks.iter().zip(heights).zip(&mut rows) {
        let window = height.saturating_sub(*pinned as u32) as usize;
        let rail = block.kind.scrolls() && block_rows.len().saturating_sub(*pinned) > window;
        if rail {
            *block_rows = self::block_rows(block, content - COLUMN).0;
        }
        placed.push(LaidBlock {
            id: block.id,
            top,
            lines: height,
            pinned: *pinned,
            rows: std::mem::take(block_rows),
            rail,
        });
        top += height;
    }
    Laid {
        blocks: placed,
        lines,
    }
}

/// Lines each block takes: the fixed ones first, held to what is left, then
/// the rest shared out evenly, the first sharers taking the remainder.
fn heights(blocks: &[Block], lines: u32) -> Vec<u32> {
    let mut left = lines;
    let mut heights = blocks
        .iter()
        .map(|block| match block.height {
            Height::Lines(wanted) => {
                let taken = wanted.min(left);
                left -= taken;
                Some(taken)
            }
            Height::Share => None,
        })
        .collect::<Vec<_>>();
    let sharers = heights.iter().filter(|height| height.is_none()).count() as u32;
    let mut index = 0;
    for height in &mut heights {
        if height.is_none() {
            *height = Some(left / sharers + u32::from(index < left % sharers));
            index += 1;
        }
    }
    heights.into_iter().map(Option::unwrap_or_default).collect()
}

/// A block's rows across `width` ems, and how many of them stay pinned.
fn block_rows(block: &Block, width: f32) -> (Vec<LaidRow>, usize) {
    match &block.kind {
        Kind::Text(spans) => (text_rows(spans, width), 0),
        Kind::Form(rows) => (
            rows.iter()
                .map(|row| form_row(row.id, &row.label, &row.control, width))
                .collect(),
            0,
        ),
        Kind::Table(table) => table_rows(table, width),
        Kind::Tree(nodes) => (nodes.iter().map(|node| tree_row(node, width)).collect(), 0),
        Kind::Bar(items) => {
            let mut x = PAD_X;
            let mut row = Vec::with_capacity(items.len());
            for item in items {
                let room = (PAD_X + width - x).max(0.0);
                let item_width = item.control.width(room);
                row.push(Item::Control {
                    id: item.id,
                    x,
                    width: item_width,
                    control: item.control.clone(),
                });
                x += item_width + COLUMN;
            }
            (vec![LaidRow::of(row)], 0)
        }
    }
}

fn text_rows(spans: &[Span], width: f32) -> Vec<LaidRow> {
    let mut rows = Vec::new();
    for span in spans {
        for line in text::wrap(&span.text, width) {
            let items = if line.is_empty() {
                Vec::new()
            } else {
                vec![Item::Text {
                    width: text::width(&line),
                    text:  SmolStr::new(line),
                    x:     PAD_X,
                    tone:  span.tone,
                    link:  span.link,
                }]
            };
            rows.push(LaidRow::of(items));
        }
    }
    rows
}

fn shown(content: &str, room: f32, x: f32, tone: Tone) -> Item {
    let fitted = text::truncate(content, room);
    Item::Text {
        width: text::width(&fitted),
        text: SmolStr::new(fitted),
        x,
        tone,
        link: None,
    }
}

fn form_row(id: u32, label: &str, control: &Control, width: f32) -> LaidRow {
    let label_width = width * LABEL_SHARE;
    let x = PAD_X + label_width;
    let room = width - label_width;
    LaidRow::of(vec![
        shown(label, label_width - COLUMN, PAD_X, Tone::Body),
        Item::Control {
            id,
            x,
            width: control.width(room),
            control: control.clone(),
        },
    ])
}

fn table_rows(table: &Table, width: f32) -> (Vec<LaidRow>, usize) {
    let cells = |cells: &[SmolStr], tone: Tone| {
        let mut x = PAD_X;
        let mut items = Vec::new();
        for (cell, column) in cells.iter().zip(&table.columns) {
            let column_width = (column.columns as f32 * COLUMN).min(PAD_X + width - x);
            if column_width <= COLUMN {
                break;
            }
            items.push(shown(cell, column_width - COLUMN, x, tone));
            x += column_width;
        }
        items
    };

    let mut rows = Vec::with_capacity(table.rows.len() + 1);
    let pinned = usize::from(!table.columns.is_empty());
    if pinned == 1 {
        let titles = table
            .columns
            .iter()
            .map(|column| column.title.clone())
            .collect::<Vec<_>>();
        rows.push(LaidRow::of(cells(&titles, Tone::Muted)));
    }
    for row in &table.rows {
        rows.push(LaidRow {
            items:  cells(&row.cells, Tone::Body),
            select: Some(Select {
                id:       row.id,
                selected: row.selected,
            }),
        });
    }
    (rows, pinned)
}

fn tree_row(node: &Node, width: f32) -> LaidRow {
    let x = PAD_X + node.depth as f32 * INDENT;
    let mut items = Vec::new();
    if let Some(open) = node.open {
        items.push(Item::Fold {
            id: node.id,
            x,
            width: FOLD_EM,
            open,
        });
    }
    let label_x = x + FOLD_EM;
    items.push(shown(
        &node.label,
        (PAD_X + width - label_x).max(0.0),
        label_x,
        Tone::Body,
    ));
    LaidRow {
        items,
        select: Some(Select {
            id:       node.id,
            selected: node.selected,
        }),
    }
}
