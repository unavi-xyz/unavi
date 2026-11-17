//! The blocks a prim's pane shows: its transform to nudge, a tree of its
//! components, a tree of the hierarchy around it, the layers its values come
//! from, and what can be done to it.

use super::{
    Inspection,
    attributes::{
        Card,
        TRANSFORM,
        cards,
    },
    edit::uniform_scale,
    format::{
        author_of,
        clip,
        document,
        layer_name,
    },
    row,
};
use crate::{
    unavi::vui::api::{
        ActionControl,
        BarItem,
        Block,
        BlockHeight,
        BlockKind,
        Control,
        FormRow,
        NumberControl,
        Pane,
        Span,
        Tone,
        TreeNode,
    },
    wired::{
        core::error::Error,
        scene::document::LayerSource,
    },
};

/// The ids blocks keep their scroll by.
mod block {
    pub const FORM: u32 = 1;
    pub const COMPONENTS_HEADING: u32 = 2;
    pub const COMPONENTS: u32 = 3;
    pub const HIERARCHY_HEADING: u32 = 4;
    pub const HIERARCHY: u32 = 5;
    pub const LAYERS: u32 = 6;
    pub const ACTIONS: u32 = 7;
}

/// Lines a tree or a table shows before it scrolls.
const WINDOW: u32 = 8;
const LAYER_WINDOW: u32 = 5;
/// Bytes of a card's heading or field shown.
const MAX_ROW: usize = 128;
/// How much a transform control changes per column of the pane the pointer
/// slides across: a sweep across the pane moves about three quarters of a
/// metre, turns about 300° or adds one and a half to the scale.
const METRES_PER_COLUMN: f32 = 0.02;
const DEGREES_PER_COLUMN: f32 = 8.0;
const SCALE_PER_COLUMN: f32 = 0.04;

fn span(text: &str, tone: Tone) -> Span {
    Span {
        text: text.to_owned(),
        tone,
        link: None,
    }
}

fn heading(id: u32, text: &str) -> Block {
    Block {
        id,
        height: BlockHeight::Lines(1),
        kind: BlockKind::Text(vec![span(text, Tone::Muted)]),
    }
}

fn number(id: u32, label: &str, value: f32, per_column: f32) -> FormRow {
    FormRow {
        id,
        label: label.to_owned(),
        control: Control::Number(NumberControl { value, per_column }),
    }
}

/// A tree that shows its nodes up to the window, then scrolls.
fn tree(id: u32, nodes: Vec<TreeNode>) -> Block {
    Block {
        id,
        height: BlockHeight::Lines((nodes.len() as u32).min(WINDOW)),
        kind: BlockKind::Tree(nodes),
    }
}

impl Inspection {
    /// The prim's components as cards.
    pub(super) fn cards(&self) -> Vec<Card> {
        let stated: Vec<_> = self
            .hit_doc
            .keys(self.hit_prim)
            .into_iter()
            .filter_map(|key| {
                let property = self.hit_doc.get(self.hit_prim, &key)?;
                Some((key, property))
            })
            .collect();
        cards(&stated)
    }

    /// What the pane is titled: the prim's name.
    #[must_use]
    pub fn title(&self) -> String {
        clip(&self.label(self.hit_prim), MAX_ROW)
    }

    /// The pane's status line: the document the prim is in, who authors it
    /// and how it is placed.
    #[must_use]
    pub fn status(&self) -> String {
        format!(
            "{} · {} · {}",
            document(&self.hit_doc),
            author_of(&self.hit_doc),
            self.kind()
        )
    }

    /// Sets what the pane says: its title, its status line and its blocks.
    pub fn fill(&mut self, pane: &Pane) -> Result<(), Error> {
        pane.set_title(&self.title())?;
        pane.set_status(Some(&self.status()))?;
        pane.set_blocks(&self.blocks())
    }

    /// Every block the pane shows, top to bottom.
    fn blocks(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        let form = self.form();
        if !form.is_empty() {
            blocks.push(Block {
                id:     block::FORM,
                height: BlockHeight::Lines(form.len() as u32),
                kind:   BlockKind::Form(form),
            });
        }
        let components = self.components();
        if !components.is_empty() {
            blocks.push(heading(block::COMPONENTS_HEADING, "Components"));
            blocks.push(tree(block::COMPONENTS, components));
        }
        let hierarchy = self.hierarchy();
        blocks.push(heading(block::HIERARCHY_HEADING, "Hierarchy"));
        blocks.push(tree(block::HIERARCHY, hierarchy));
        if let Some(table) = self.layers() {
            blocks.push(Block {
                id:     block::LAYERS,
                height: BlockHeight::Lines((table.rows.len() as u32 + 1).min(LAYER_WINDOW)),
                kind:   BlockKind::Table(table),
            });
        }
        blocks.push(Block {
            id:     block::ACTIONS,
            height: BlockHeight::Lines(1),
            kind:   BlockKind::Bar(self.actions()),
        });
        blocks
    }

    /// The lock, and the transform's parts, each nudged in place.
    fn form(&self) -> Vec<FormRow> {
        let mut rows = Vec::new();
        if self.hit_doc.may_write() {
            let here = self.locked_here();
            rows.push(if !here && self.locked() {
                // Only the prim that states a lock can lift it.
                FormRow {
                    id:      row::LOCKED,
                    label:   "Locked".to_owned(),
                    control: Control::Value("from above".to_owned()),
                }
            } else {
                FormRow {
                    id:      row::LOCKED,
                    label:   "Locked".to_owned(),
                    control: Control::Toggle(here),
                }
            });
        }
        if !self.editable() {
            return rows;
        }
        let Some(pose) = self.held_pose().or_else(|| self.pose()) else {
            return rows;
        };
        let (yaw, ..) = glam::Quat::from(pose.rotation).to_euler(glam::EulerRot::YXZ);
        let t = pose.translation;
        for (index, (label, value)) in [
            ("Position X", t.x),
            ("Position Y", t.y),
            ("Position Z", t.z),
        ]
        .into_iter()
        .enumerate()
        {
            rows.push(number(
                row::POSITION + index as u32,
                label,
                value,
                METRES_PER_COLUMN,
            ));
        }
        rows.push(number(
            row::TURN,
            "Turn",
            yaw.to_degrees(),
            DEGREES_PER_COLUMN,
        ));
        rows.push(number(
            row::SCALE,
            "Scale",
            uniform_scale(pose.scale),
            SCALE_PER_COLUMN,
        ));
        rows
    }

    /// The components as a tree: a node per card, folded to its summary or
    /// open to its fields. The transform has a form of its own when it can be
    /// edited.
    fn components(&self) -> Vec<TreeNode> {
        let mut nodes = Vec::new();
        for (index, card) in self.cards().iter().enumerate() {
            if card.name == TRANSFORM && self.editable() {
                continue;
            }
            let id = row::CARD + index as u32 * row::CARD_STRIDE;
            let open = self.open.contains(&card.name);
            let source = self
                .hit_doc
                .source(self.hit_prim, &card.key)
                .filter(|source| !matches!(source, LayerSource::Document))
                .map(|source| format!(" · {}", layer_name(&source)))
                .unwrap_or_default();
            let label = if open {
                format!("{}{source}", card.name)
            } else {
                format!("{}  {}{source}", card.name, card.summary)
            };
            nodes.push(TreeNode {
                id,
                label: clip(&label, MAX_ROW),
                depth: 0,
                open: Some(open),
                selected: false,
            });
            if !open {
                continue;
            }
            let fields = card.fields.iter().take(row::CARD_STRIDE as usize - 1);
            for (offset, (label, value)) in fields.enumerate() {
                nodes.push(TreeNode {
                    id:       id + 1 + offset as u32,
                    label:    clip(&format!("{label}  {value}"), MAX_ROW),
                    depth:    1,
                    open:     None,
                    selected: false,
                });
            }
        }
        nodes
    }

    /// What can be done to the prim.
    fn actions(&self) -> Vec<BarItem> {
        let action = |id, label: &str| BarItem {
            id,
            control: Control::Action(ActionControl {
                label:   label.to_owned(),
                primary: false,
            }),
        };
        let mut items = Vec::new();
        if self.site.is_some() && self.placed_in.may_write() {
            items.push(action(row::MAKE_LOCAL, "Make local"));
        }
        if self.hit_doc.may_write() && !self.locked() {
            items.push(action(row::DELETE, "Delete"));
        }
        items.push(action(row::CLOSE, "Close"));
        items
    }

    fn kind(&self) -> &'static str {
        if self.loose {
            "loose"
        } else if self.placed_in.id() == self.hit_doc.id() {
            "space"
        } else {
            "linked"
        }
    }
}
