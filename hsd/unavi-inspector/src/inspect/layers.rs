//! The layers the prim inspected takes its values from, as a table: each
//! layer and how many of the prim's values it states.

use super::{
    Inspection,
    format::{
        layer_name,
        layer_rank,
    },
    row,
};
use crate::unavi::vui::api::{
    Table,
    TableColumn,
    TableRow,
};

/// Columns the layer's name and the count take, which together fill a pane of
/// 40 columns less its margins and rail.
const NAME_COLUMNS: u32 = 28;
const COUNT_COLUMNS: u32 = 8;

impl Inspection {
    /// One row per layer that states a value of the prim, or `None` when no
    /// layer states any.
    pub(super) fn layers(&self) -> Option<Table> {
        let mut counts: Vec<((u8, String), usize)> = Vec::new();
        for key in self.hit_doc.keys(self.hit_prim) {
            let Some(source) = self.hit_doc.source(self.hit_prim, &key) else {
                continue;
            };
            let layer = (layer_rank(&source), layer_name(&source));
            match counts.iter_mut().find(|(seen, _)| *seen == layer) {
                Some((_, count)) => *count += 1,
                None => counts.push((layer, 1)),
            }
        }
        if counts.is_empty() {
            return None;
        }
        counts.sort();
        let rows = counts
            .into_iter()
            .enumerate()
            .map(|(index, ((_, name), count))| TableRow {
                id:       row::LAYER + index as u32,
                cells:    vec![name, count.to_string()],
                selected: false,
            })
            .collect();
        Some(Table {
            columns: vec![
                TableColumn {
                    title:   "Layer".to_owned(),
                    columns: NAME_COLUMNS,
                },
                TableColumn {
                    title:   "Values".to_owned(),
                    columns: COUNT_COLUMNS,
                },
            ],
            rows,
        })
    }
}
