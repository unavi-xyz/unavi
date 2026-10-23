//! The rows the slate shows for a prim: a fixed head and foot, and between
//! them what the selected tab lists.

use super::{
    Inspection,
    MAX_CHILDREN,
    attributes::lines,
    edit::uniform_scale,
    format::{
        action,
        author_of,
        document_name,
        field,
        header,
        name,
    },
    row,
};
use crate::{
    tab::Tab,
    unavi::vui::api::{
        Number,
        Row,
        RowKind,
        Tabs,
    },
};

/// How much a transform row changes per metre the pointer slides across the
/// slate: a sweep across its width moves about half a metre, turns about 160°
/// or adds one to the scale.
const METRES_PER_METRE: f32 = 1.0;
const DEGREES_PER_METRE: f32 = 360.0;
const SCALE_PER_METRE: f32 = 2.0;

/// Rows a slate holds, less the head (name, source, tabs) and the Close row,
/// and one for the line saying how many more there are.
const MAX_ATTRIBUTES: usize = 24 - 4 - 1;

impl Inspection {
    #[must_use]
    pub fn rows(&self) -> Vec<Row> {
        let mut rows = vec![
            header(row::NAME, &name(&self.hit_doc, self.hit_prim)),
            field(
                row::SOURCE,
                "Source",
                &format!(
                    "{} · {} · {}",
                    document_name(&self.hit_doc),
                    author_of(&self.hit_doc),
                    self.kind()
                ),
            ),
            Row {
                id:    row::TABS,
                label: String::new(),
                kind:  RowKind::Tabs(Tabs {
                    labels:   Tab::ALL.iter().map(|tab| tab.label().to_owned()).collect(),
                    selected: self.tab.index(),
                }),
            },
        ];
        match self.tab {
            Tab::Transform => self.transform_rows(&mut rows),
            Tab::Tree => self.tree_rows(&mut rows),
            Tab::Attributes => self.attribute_rows(&mut rows),
        }
        rows.push(action(row::CLOSE, "Close"));
        rows
    }

    fn transform_rows(&self, rows: &mut Vec<Row>) {
        if self.hit_doc.may_write() {
            let here = self.locked_here();
            rows.push(if !here && self.locked() {
                // Only the prim that states a lock can lift it.
                field(row::LOCKED, "Locked", "by a prim above")
            } else {
                Row {
                    id:    row::LOCKED,
                    label: "Locked".to_owned(),
                    kind:  RowKind::Toggle(here),
                }
            });
        }
        if let Some(pose) = self.held_pose().or_else(|| self.pose()) {
            let editable = self.editable();
            let (yaw, ..) = glam::Quat::from(pose.rotation).to_euler(glam::EulerRot::YXZ);
            let t = pose.translation;
            for (id, label, value, per_metre) in [
                (row::POSITION, "Position X", t.x, METRES_PER_METRE),
                (row::POSITION + 1, "Position Y", t.y, METRES_PER_METRE),
                (row::POSITION + 2, "Position Z", t.z, METRES_PER_METRE),
                (row::TURN, "Turn", yaw.to_degrees(), DEGREES_PER_METRE),
                (
                    row::SCALE,
                    "Scale",
                    uniform_scale(pose.scale),
                    SCALE_PER_METRE,
                ),
            ] {
                rows.push(if editable {
                    Row {
                        id,
                        label: label.to_owned(),
                        kind: RowKind::Number(Number { value, per_metre }),
                    }
                } else {
                    field(id, label, &format!("{value:.2}"))
                });
            }
        }
        if self.site.is_some() && self.placed_in.may_write() {
            rows.push(action(row::MAKE_LOCAL, "Make local"));
        }
        if self.hit_doc.may_write() && !self.locked() {
            rows.push(action(row::DELETE, "Delete"));
        }
    }

    /// The parent and the children that have a dot, each a row that inspects
    /// it.
    fn tree_rows(&self, rows: &mut Vec<Row>) {
        let parent = self.parent();
        let children = self.hit_doc.children(self.hit_prim);
        if parent.is_none() && children.is_empty() {
            rows.push(field(row::PARENT, "Tree", "nothing above or beneath"));
            return;
        }
        if let Some(parent) = parent {
            rows.push(action(
                row::PARENT,
                &format!("Up to {}", name(&self.hit_doc, parent)),
            ));
        }
        for (index, child) in children.iter().take(MAX_CHILDREN).enumerate() {
            rows.push(action(
                row::CHILD + index as u32,
                &format!("Into {}", name(&self.hit_doc, *child)),
            ));
        }
        if children.len() > MAX_CHILDREN {
            rows.push(field(
                row::MORE,
                "More",
                &format!("{} more beneath", children.len() - MAX_CHILDREN),
            ));
        }
    }

    /// A line per thing the prim states, each saying which layer it comes
    /// from when that is not the document itself.
    fn attribute_rows(&self, rows: &mut Vec<Row>) {
        let stated: Vec<_> = self
            .hit_doc
            .keys(self.hit_prim)
            .into_iter()
            .filter_map(|key| {
                let property = self.hit_doc.get(self.hit_prim, &key)?;
                Some((key, property))
            })
            .collect();
        let lines = lines(&stated);
        for (index, line) in lines.iter().take(MAX_ATTRIBUTES).enumerate() {
            let value = self.source_of(&line.key).map_or_else(
                || line.value.clone(),
                |source| format!("{} · {source}", line.value),
            );
            rows.push(field(row::ATTRIBUTE + index as u32, &line.label, &value));
        }
        if lines.len() > MAX_ATTRIBUTES {
            rows.push(field(
                row::MORE,
                "More",
                &format!("{} more", lines.len() - MAX_ATTRIBUTES),
            ));
        }
    }

    fn kind(&self) -> &'static str {
        if self.loose {
            "loose"
        } else if self.placed_in.id() == self.hit_doc.id() {
            "this space"
        } else {
            "referenced"
        }
    }
}
