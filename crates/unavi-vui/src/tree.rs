use smol_str::SmolStr;

use crate::mote::{
    MoteSpec,
    Role,
};

pub struct Node {
    pub spec:     MoteSpec,
    pub children: Vec<Self>,
}

impl Node {
    fn new(role: Role, label: &str, children: Vec<Self>) -> Self {
        Self {
            spec: MoteSpec {
                role,
                label: SmolStr::new(label),
                description: None,
            },
            children,
        }
    }

    /// Activating it does something, and it cannot be taken out.
    #[must_use]
    pub fn action(label: &str) -> Self {
        Self::new(Role::Action, label, Vec::new())
    }

    /// A thing rather than a command: it can be pulled out of the orbit and
    /// put somewhere, which is what makes a drag mean anything.
    #[must_use]
    pub fn item(label: &str) -> Self {
        Self::new(Role::Item, label, Vec::new())
    }

    /// Consequential: it opens a cast site rather than firing on release.
    #[must_use]
    pub fn cast(label: &str) -> Self {
        Self::new(Role::Cast, label, Vec::new())
    }

    #[must_use]
    pub fn group(label: &str, children: Vec<Self>) -> Self {
        let groups = children
            .iter()
            .filter(|child| matches!(child.spec.role, Role::Group { .. }))
            .count();
        Self::new(
            Role::Group {
                children: children.len(),
                groups,
            },
            label,
            children,
        )
    }

    /// What this one does, shown on its placard once attention has been held.
    #[must_use]
    pub fn describe(mut self, description: &str) -> Self {
        self.spec.description = Some(SmolStr::new(description));
        self
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
                role:        Role::Parent {
                    depth: self.depth(),
                },
                label:       self.here(),
                description: Some(SmolStr::new_static("The level you are inside.")),
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
            Role::Group { .. } => {
                self.path.push(index);
                Navigation::Bloomed(label)
            }
            Role::Cast => Navigation::Cast(label),
            Role::Action | Role::Item | Role::Parent { .. } => Navigation::Activated(label),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> Tree {
        Tree::new(Node::group(
            "Root",
            vec![
                Node::cast("Home"),
                Node::group("Places", vec![Node::action("Atrium"), Node::action("Club")]),
                Node::action("Lens"),
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
    fn the_parent_mote_carries_the_current_depth() {
        let mut tree = tree();
        tree.select(1);
        assert_eq!(tree.level()[0].role, Role::Parent { depth: 1 });
    }

    #[test]
    fn a_branch_counts_how_many_children_are_themselves_containers() {
        let node = Node::group(
            "mixed",
            vec![
                Node::group("a", vec![Node::action("x")]),
                Node::action("y"),
                Node::group("b", vec![]),
            ],
        );
        assert_eq!(
            node.spec.role,
            Role::Group {
                children: 3,
                groups:   2,
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
        let mut node = Node::action("bottom");
        for level in 0..32 {
            node = Node::group(&format!("level{level}"), vec![node]);
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
