//! The rows a prim's slate shows: what it is and which document it is in,
//! then a card per component, folded to a summary or open to its fields, the
//! tree around it, and what can be done to it.

use super::{
    Inspection,
    MAX_CHILDREN,
    TREE,
    attributes::{
        Card,
        TRANSFORM,
        cards,
    },
    edit::uniform_scale,
    format::{
        action,
        author_of,
        document,
        field,
        header,
        name,
        provenance,
        row as plain,
    },
    row,
};
use crate::unavi::vui::api::{
    Number,
    Row,
    RowKind,
    Section,
    Vector,
};

/// Most rows a slate holds.
const MAX_ROWS: usize = 24;
/// How much a transform row changes per metre the pointer slides across the
/// slate: a sweep across its width moves about half a metre, turns about 160°
/// or adds one to the scale.
const METRES_PER_METRE: f32 = 1.0;
const DEGREES_PER_METRE: f32 = 360.0;
const SCALE_PER_METRE: f32 = 2.0;

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

    #[must_use]
    pub fn rows(&self) -> Vec<Row> {
        let (doc_label, doc_color) = document(&self.hit_doc);
        let title = name(&self.hit_doc, self.hit_prim).unwrap_or_else(|| {
            if self.parent().is_none() {
                "Root".to_owned()
            } else {
                "Prim".to_owned()
            }
        });
        let mut rows = vec![
            header(row::NAME, &title),
            Row {
                dot: Some(doc_color),
                ..field(
                    row::DOCUMENT,
                    &doc_label,
                    &format!("{} · {}", author_of(&self.hit_doc), self.kind()),
                )
            },
        ];
        if self.hit_doc.may_write() {
            let here = self.locked_here();
            rows.push(if !here && self.locked() {
                // Only the prim that states a lock can lift it.
                field(row::LOCKED, "Locked", "from above")
            } else {
                plain(row::LOCKED, "Locked", RowKind::Toggle(here))
            });
        }
        for (index, card) in self.cards().iter().enumerate() {
            self.card_rows(index as u32, card, &mut rows);
        }
        self.tree_rows(&mut rows);

        let mut foot = Vec::new();
        if self.site.is_some() && self.placed_in.may_write() {
            foot.push(action(row::MAKE_LOCAL, "Make local"));
        }
        if self.hit_doc.may_write() && !self.locked() {
            foot.push(action(row::DELETE, "Delete"));
        }
        foot.push(action(row::CLOSE, "Close"));
        rows.truncate(MAX_ROWS - foot.len());
        rows.extend(foot);
        rows
    }

    fn card_rows(&self, index: u32, card: &Card, rows: &mut Vec<Row>) {
        let id = row::CARD + index * row::CARD_STRIDE;
        let open = self.open.contains(&card.name);
        rows.push(Row {
            mark: provenance(self.hit_doc.source(self.hit_prim, &card.key).as_ref()),
            ..plain(
                id,
                &card.name,
                RowKind::Section(Section {
                    open,
                    summary: if open {
                        String::new()
                    } else {
                        card.summary.clone()
                    },
                }),
            )
        });
        if !open {
            return;
        }
        if card.name == TRANSFORM && self.editable() {
            self.transform_rows(rows);
            return;
        }
        let fields = card.fields.iter().take(row::CARD_STRIDE as usize - 1);
        for (offset, (label, value)) in fields.enumerate() {
            rows.push(field(id + 1 + offset as u32, label, value));
        }
    }

    /// The transform's parts, each scrubbed in place.
    fn transform_rows(&self, rows: &mut Vec<Row>) {
        let Some(pose) = self.held_pose().or_else(|| self.pose()) else {
            return;
        };
        let (yaw, ..) = glam::Quat::from(pose.rotation).to_euler(glam::EulerRot::YXZ);
        let t = pose.translation;
        rows.push(plain(
            row::POSITION,
            "Position",
            RowKind::Vector(Vector {
                values:    (t.x, t.y, t.z),
                per_metre: METRES_PER_METRE,
            }),
        ));
        for (id, label, value, per_metre) in [
            (row::TURN, "Turn", yaw.to_degrees(), DEGREES_PER_METRE),
            (
                row::SCALE,
                "Scale",
                uniform_scale(pose.scale),
                SCALE_PER_METRE,
            ),
        ] {
            rows.push(plain(
                id,
                label,
                RowKind::Number(Number { value, per_metre }),
            ));
        }
    }

    /// The tree around the prim, as a card: its parent, then its children.
    fn tree_rows(&self, rows: &mut Vec<Row>) {
        let parent = self.parent();
        let children = self.hit_doc.children(self.hit_prim);
        let open = self.open.contains(TREE);
        let summary = match (parent.is_some(), children.len()) {
            (false, 0) => "alone".to_owned(),
            (_, 0) => "no children".to_owned(),
            (_, 1) => "1 child".to_owned(),
            (_, count) => format!("{count} children"),
        };
        rows.push(plain(
            row::TREE,
            TREE,
            RowKind::Section(Section {
                open,
                summary: if open { String::new() } else { summary },
            }),
        ));
        if !open {
            return;
        }
        let label = |prim| name(&self.hit_doc, prim).unwrap_or_else(|| "Prim".to_owned());
        if let Some(parent) = parent {
            rows.push(action(row::PARENT, &format!("Parent · {}", label(parent))));
        }
        for (index, child) in children.iter().take(MAX_CHILDREN).enumerate() {
            rows.push(action(row::CHILD + index as u32, &label(*child)));
        }
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
