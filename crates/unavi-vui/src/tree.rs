use smol_str::SmolStr;

use crate::mote::{
    Grab,
    MoteKind,
    MoteSpec,
    Role,
};

pub struct Node {
    pub spec:     MoteSpec,
    pub children: Vec<Self>,
}

impl Node {
    /// A command: activating it does something, and it cannot be taken out.
    #[must_use]
    pub fn leaf(kind: MoteKind, label: &str) -> Self {
        Self {
            spec:     MoteSpec {
                kind,
                role: Role::Leaf,
                label: SmolStr::new(label),
                grab: Grab::Fixed,
                embodied: false,
            },
            children: Vec::new(),
        }
    }

    /// A thing rather than a command: it can be pulled out of the orbit and
    /// put somewhere, which is what makes a drag mean anything.
    #[must_use]
    pub fn takeable(kind: MoteKind, label: &str) -> Self {
        let mut node = Self::leaf(kind, label);
        node.spec.grab = Grab::Takeable;
        node
    }

    #[must_use]
    pub fn cast(kind: MoteKind, label: &str) -> Self {
        Self {
            spec:     MoteSpec {
                kind,
                role: Role::Cast,
                label: SmolStr::new(label),
                grab: Grab::Fixed,
                embodied: false,
            },
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn branch(kind: MoteKind, label: &str, children: Vec<Self>) -> Self {
        let folders = children
            .iter()
            .filter(|child| matches!(child.spec.role, Role::Branch { .. }))
            .count();
        Self {
            spec: MoteSpec {
                kind,
                role: Role::Branch {
                    children: children.len(),
                    folders,
                },
                label: SmolStr::new(label),
                grab: Grab::Fixed,
                embodied: false,
            },
            children,
        }
    }
}

/// What a selection did, for the caller to report or react to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Navigation {
    Collapsed,
    Bloomed(SmolStr),
    Activated(SmolStr),
    Cast(SmolStr),
    None,
}

/// A tree and the path currently bloomed within it.
///
/// Depth is unbounded. The one structural rule is that **slot 0 is the parent
/// bead whenever the path is non-empty, and nothing else is ever placed
/// there** — the way back is in the same position at every level, which is
/// what makes it learnable.
pub struct Tree {
    root: Node,
    path: Vec<usize>,
}

impl Tree {
    #[must_use]
    pub const fn new(root: Node) -> Self {
        Self {
            root,
            path: Vec::new(),
        }
    }

    #[must_use]
    pub const fn depth(&self) -> usize {
        self.path.len()
    }

    #[must_use]
    pub const fn is_nested(&self) -> bool {
        !self.path.is_empty()
    }

    #[must_use]
    pub fn path(&self) -> &[usize] {
        &self.path
    }

    fn node(&self) -> Option<&Node> {
        let mut node = &self.root;
        for &index in &self.path {
            node = node.children.get(index)?;
        }
        Some(node)
    }

    /// The label of the level currently open.
    #[must_use]
    pub fn here(&self) -> SmolStr {
        self.node()
            .map_or_else(|| SmolStr::new_static("?"), |node| node.spec.label.clone())
    }

    /// Motes at the current level, parent bead first when nested.
    #[must_use]
    pub fn level(&self) -> Vec<MoteSpec> {
        let Some(node) = self.node() else {
            return Vec::new();
        };
        let mut specs = Vec::with_capacity(node.children.len() + 1);
        if self.is_nested() {
            specs.push(MoteSpec {
                kind:     MoteKind::Folder,
                role:     Role::Parent {
                    depth: self.depth(),
                },
                label:    self.here(),
                grab:     Grab::Fixed,
                embodied: false,
            });
        }
        specs.extend(node.children.iter().map(|child| child.spec.clone()));
        specs
    }

    pub fn select(&mut self, slot: usize) -> Navigation {
        if self.is_nested() && slot == 0 {
            self.path.pop();
            return Navigation::Collapsed;
        }
        let index = slot - usize::from(self.is_nested());
        let Some(child) = self.node().and_then(|node| node.children.get(index)) else {
            return Navigation::None;
        };
        let label = child.spec.label.clone();
        match child.spec.role {
            Role::Branch { .. } => {
                self.path.push(index);
                Navigation::Bloomed(label)
            }
            Role::Cast => Navigation::Cast(label),
            Role::Leaf | Role::Parent { .. } => Navigation::Activated(label),
        }
    }

    /// Collapses every level at once. Reached by pulling the parent mote out
    /// of the orbit rather than tapping it, so one body means both "up one"
    /// and "all the way out" without a second control.
    pub fn reset(&mut self) -> Navigation {
        if self.path.is_empty() {
            return Navigation::None;
        }
        self.path.clear();
        Navigation::Collapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Tree {
        Tree::new(Node::branch(
            MoteKind::Folder,
            "Root",
            vec![
                Node::cast(MoteKind::Command, "Home"),
                Node::branch(
                    MoteKind::Space,
                    "Places",
                    vec![
                        Node::leaf(MoteKind::Space, "Atrium"),
                        Node::leaf(MoteKind::Space, "Club"),
                    ],
                ),
                Node::leaf(MoteKind::Tool, "Lens"),
            ],
        ))
    }

    fn labels(tree: &Tree) -> Vec<String> {
        tree.level()
            .into_iter()
            .map(|spec| spec.label.to_string())
            .collect()
    }

    #[test]
    fn the_root_level_has_no_parent_bead() {
        let tree = tree();
        assert_eq!(tree.depth(), 0);
        assert_eq!(labels(&tree), vec!["Home", "Places", "Lens"]);
    }

    #[test]
    fn blooming_puts_the_parent_at_slot_zero() {
        let mut tree = tree();
        assert_eq!(tree.select(1), Navigation::Bloomed("Places".into()));
        assert_eq!(labels(&tree), vec!["Places", "Atrium", "Club"]);
        assert_eq!(tree.level()[0].role, Role::Parent { depth: 1 });
    }

    #[test]
    fn slot_zero_collapses_only_when_nested() {
        let mut tree = tree();
        assert_eq!(tree.select(0), Navigation::Cast("Home".into()));
        assert_eq!(
            tree.depth(),
            0,
            "slot 0 at the root is a child, not a way back"
        );

        tree.select(1);
        assert_eq!(tree.select(0), Navigation::Collapsed);
        assert_eq!(tree.depth(), 0);
        assert_eq!(labels(&tree), vec!["Home", "Places", "Lens"]);
    }

    #[test]
    fn leaves_activate_without_moving() {
        let mut tree = tree();
        assert_eq!(tree.select(2), Navigation::Activated("Lens".into()));
        assert_eq!(tree.depth(), 0);
    }

    #[test]
    fn the_parent_bead_is_named_for_the_level_it_leaves() {
        let mut tree = tree();
        tree.select(1);
        assert_eq!(tree.here(), "Places");
        assert_eq!(tree.level()[0].label, "Places");
    }

    #[test]
    fn reset_collapses_every_level_at_once() {
        let mut tree = tree();
        tree.select(1);
        assert_eq!(tree.depth(), 1);
        assert_eq!(tree.reset(), Navigation::Collapsed);
        assert_eq!(tree.depth(), 0);
        assert_eq!(labels(&tree), vec!["Home", "Places", "Lens"]);
    }

    #[test]
    fn reset_at_the_root_is_a_no_op() {
        let mut tree = tree();
        assert_eq!(tree.reset(), Navigation::None);
        assert_eq!(tree.depth(), 0);
    }

    #[test]
    fn the_parent_mote_carries_the_current_depth() {
        let mut tree = tree();
        tree.select(1);
        assert_eq!(tree.level()[0].role, Role::Parent { depth: 1 });
    }

    #[test]
    fn a_branch_counts_how_many_children_are_themselves_containers() {
        let node = Node::branch(
            MoteKind::Folder,
            "mixed",
            vec![
                Node::branch(MoteKind::Folder, "a", vec![Node::leaf(MoteKind::Item, "x")]),
                Node::leaf(MoteKind::Item, "y"),
                Node::branch(MoteKind::Folder, "b", vec![]),
            ],
        );
        assert_eq!(
            node.spec.role,
            Role::Branch {
                children: 3,
                folders:  2,
            }
        );
    }

    #[test]
    fn selecting_out_of_range_does_nothing() {
        let mut tree = tree();
        assert_eq!(tree.select(99), Navigation::None);
        assert_eq!(tree.depth(), 0);
    }

    #[test]
    fn depth_is_unbounded() {
        let mut node = Node::leaf(MoteKind::Item, "bottom");
        for level in 0..32 {
            node = Node::branch(MoteKind::Folder, &format!("level{level}"), vec![node]);
        }
        let mut tree = Tree::new(node);
        let mut bloomed = 0;
        while matches!(
            tree.select(usize::from(tree.is_nested())),
            Navigation::Bloomed(_)
        ) {
            bloomed += 1;
        }
        // 32 nested branches, so 31 of them can be descended into before the
        // leaf at the bottom activates instead.
        assert_eq!(bloomed, 31);
        assert_eq!(tree.depth(), 31);
    }
}
