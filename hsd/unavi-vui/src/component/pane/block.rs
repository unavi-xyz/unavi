//! The blocks a pane stacks, as plain data.

use smol_str::SmolStr;

use crate::component::pane::control::Control;

/// How a line of text is set.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Body,
    /// Ink at reduced contrast.
    Muted,
    Warning,
}

/// How many lines a block takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Height {
    Lines(u32),
    /// Whatever the fixed blocks leave, split evenly with every other
    /// sharing block.
    Share,
}

/// One part of a pane's body.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub id:     u32,
    pub height: Height,
    pub kind:   Kind,
}

/// What a block shows.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// Prose, wrapped to the pane.
    Text(Vec<Span>),
    /// Rows of a label and one control.
    Form(Vec<FormRow>),
    /// Rows under named columns.
    Table(Table),
    /// Indented rows, each foldable. The consumer sends the rows that are
    /// visible and sets what a fold reports.
    Tree(Vec<Node>),
    /// One line of controls.
    Bar(Vec<BarItem>),
}

impl Kind {
    /// Whether content past the block's lines scrolls.
    #[must_use]
    pub const fn scrolls(&self) -> bool {
        !matches!(self, Self::Bar(_))
    }
}

/// A paragraph of a text block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: SmolStr,
    pub tone: Tone,
    /// The id a press on the paragraph reports as followed.
    pub link: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormRow {
    /// Names the control in events.
    pub id:      u32,
    pub label:   SmolStr,
    pub control: Control,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub title:   SmolStr,
    /// Width in columns.
    pub columns: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub columns: Vec<Column>,
    pub rows:    Vec<TableRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    pub id:       u32,
    pub cells:    Vec<SmolStr>,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id:       u32,
    pub label:    SmolStr,
    pub depth:    u32,
    /// Whether it is unfolded, or none for a leaf.
    pub open:     Option<bool>,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarItem {
    pub id:      u32,
    pub control: Control,
}
