//! The prim hierarchy as the rows of a tree: from the topmost ancestor of the
//! prim inspected down, each prim folded or open.

use super::{
    Inspection,
    format::{
        clip,
        name,
    },
    row,
};
use crate::unavi::vui::api::TreeNode;

/// Most rows the tree lists.
const MAX_NODES: usize = 200;
/// Most ancestors climbed to find the top of the hierarchy, which also keeps
/// a parent cycle from looping.
const MAX_DEPTH: usize = 64;
/// Bytes of a prim's name shown.
const MAX_LABEL: usize = 64;

impl Inspection {
    /// Opens every ancestor of the prim inspected, and the prim itself, so it
    /// is on view.
    pub(super) fn unfold_to_here(&mut self) {
        let mut prim = Some(self.hit_prim);
        for _ in 0..MAX_DEPTH {
            let Some(here) = prim else {
                return;
            };
            self.unfolded.insert(here);
            prim = self.parent_of(here);
        }
    }

    /// The topmost ancestor of the prim inspected.
    fn top(&self) -> (u64, u64) {
        let mut top = self.hit_prim;
        for _ in 0..MAX_DEPTH {
            let Some(parent) = self.parent_of(top) else {
                break;
            };
            top = parent;
        }
        top
    }

    /// The rows on view, and the prim each stands for.
    pub(super) fn hierarchy(&mut self) -> Vec<TreeNode> {
        self.nodes.clear();
        let mut nodes = Vec::new();
        self.walk(self.top(), 0, &mut nodes);
        nodes
    }

    fn walk(&mut self, prim: (u64, u64), depth: u32, nodes: &mut Vec<TreeNode>) {
        if nodes.len() >= MAX_NODES {
            return;
        }
        let children = self.hit_doc.children(prim);
        let open = self.unfolded.contains(&prim);
        nodes.push(TreeNode {
            id: row::NODE + self.nodes.len() as u32,
            label: clip(&self.label(prim), MAX_LABEL),
            depth,
            open: (!children.is_empty()).then_some(open),
            selected: prim == self.hit_prim,
        });
        self.nodes.push(prim);
        if open {
            for child in children {
                self.walk(child, depth + 1, nodes);
            }
        }
    }

    /// A prim's name, else what it is in the hierarchy.
    pub(super) fn label(&self, prim: (u64, u64)) -> String {
        name(&self.hit_doc, prim).unwrap_or_else(|| {
            if self.parent_of(prim).is_none() {
                "Root".to_owned()
            } else {
                "Prim".to_owned()
            }
        })
    }
}
