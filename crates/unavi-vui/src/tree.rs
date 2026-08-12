use smol_str::SmolStr;

use crate::{
    model::Model,
    mote::{
        MoteSpec,
        Role,
    },
};

/// A static tree of motes, and the simplest thing that is a [`Model`].
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

    #[must_use]
    pub fn action(label: &str) -> Self {
        Self::new(Role::Action, label, Vec::new())
    }

    #[must_use]
    pub fn item(label: &str) -> Self {
        Self::new(Role::Item, label, Vec::new())
    }

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

    fn at(&self, path: &[usize]) -> Option<&Self> {
        let mut node = self;
        for &index in path {
            node = node.children.get(index)?;
        }
        Some(node)
    }
}

impl Model for Node {
    fn root(&self) -> MoteSpec {
        self.spec.clone()
    }

    fn children(&self, path: &[usize]) -> Vec<MoteSpec> {
        self.at(path).map_or_else(Vec::new, |node| {
            node.children.iter().map(|child| child.spec.clone()).collect()
        })
    }

    fn activate(&mut self, _path: &[usize]) {}
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

/// A path through a [`Model`], and the level currently open.
///
/// Slot 0 is the way back whenever the path is non-empty; nothing else is ever
/// placed there.
pub struct Tree<M> {
    model: M,
    path:  Vec<usize>,
}

impl<M: Model> Tree<M> {
    #[must_use]
    pub const fn new(model: M) -> Self {
        Self {
            model,
            path: Vec::new(),
        }
    }

    #[must_use]
    pub const fn model(&self) -> &M {
        &self.model
    }

    pub const fn model_mut(&mut self) -> &mut M {
        &mut self.model
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

    /// The mote standing for the level currently open.
    #[must_use]
    pub fn here(&self) -> MoteSpec {
        self.spec_at(self.depth())
            .unwrap_or_else(|| self.model.root())
    }

    /// The mote for the level `depth` levels down from the root.
    fn spec_at(&self, depth: usize) -> Option<MoteSpec> {
        let (&index, parent) = self.path.get(..depth)?.split_last()?;
        self.model.children(parent).into_iter().nth(index)
    }

    /// Motes at the current level, the way back first when nested.
    #[must_use]
    pub fn level(&self) -> Vec<MoteSpec> {
        let children = self.model.children(&self.path);
        if !self.is_nested() {
            return children;
        }
        let mut specs = Vec::with_capacity(children.len() + 1);
        specs.push(MoteSpec {
            role:        Role::Parent {
                depth: self.depth(),
            },
            label:       self.here().label,
            description: Some(SmolStr::new_static("The level you are inside.")),
        });
        specs.extend(children);
        specs
    }

    /// The levels above the one the parent mote already stands for, nearest
    /// first. This is the breadcrumb; there is no separate widget.
    #[must_use]
    pub fn trail(&self) -> Vec<MoteSpec> {
        (0..self.depth().saturating_sub(1))
            .rev()
            .map(|depth| {
                let spec = self
                    .spec_at(depth)
                    .unwrap_or_else(|| self.model.root());
                MoteSpec {
                    role: Role::Parent { depth },
                    ..spec
                }
            })
            .collect()
    }

    pub fn select(&mut self, slot: usize) -> Navigation {
        if self.is_nested() && slot == 0 {
            self.path.pop();
            return Navigation::Collapsed;
        }
        let index = slot - usize::from(self.is_nested());
        let Some(child) = self.model.children(&self.path).into_iter().nth(index) else {
            return Navigation::None;
        };
        let label = child.label;
        match child.role {
            Role::Group { .. } => {
                self.path.push(index);
                Navigation::Bloomed(label)
            }
            Role::Cast => Navigation::Cast(label),
            Role::Action | Role::Item | Role::Parent { .. } => {
                self.path.push(index);
                self.model.activate(&self.path);
                self.path.pop();
                Navigation::Activated(label)
            }
        }
    }

    /// Climbs to `depth`, which a trail mote does when it is selected.
    pub fn ascend_to(&mut self, depth: usize) {
        self.path.truncate(depth);
    }

    pub fn reset(&mut self) {
        self.path.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node() -> Node {
        Node::group(
            "Root",
            vec![
                Node::cast("Home"),
                Node::group("Places", vec![Node::action("Atrium"), Node::action("Club")]),
                Node::action("Lens"),
            ],
        )
    }

    fn tree() -> Tree<Node> {
        Tree::new(node())
    }

    fn labels(tree: &Tree<Node>) -> Vec<String> {
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
        assert_eq!(tree.here().label, "Places");
        assert_eq!(tree.level()[0].label, "Places");
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

    #[test]
    fn the_root_level_has_no_trail() {
        assert!(tree().trail().is_empty());
    }

    #[test]
    fn the_trail_holds_the_levels_the_parent_mote_does_not() {
        let mut tree = Tree::new(Node::group(
            "Root",
            vec![Node::group(
                "Orchard",
                vec![Node::group(
                    "Apples",
                    vec![Node::group("Baskets", vec![Node::item("Gala")])],
                )],
            )],
        ));
        tree.select(0);
        assert_eq!(
            tree.trail().len(),
            0,
            "one level down, the parent mote is the whole breadcrumb"
        );

        tree.select(1);
        let trail = tree.trail();
        assert_eq!(
            trail.iter().map(|spec| spec.label.as_str()).collect::<Vec<_>>(),
            vec!["Root"],
            "the level above the parent mote, and no deeper"
        );

        tree.select(1);
        let trail = tree.trail();
        assert_eq!(
            trail.iter().map(|spec| spec.label.as_str()).collect::<Vec<_>>(),
            vec!["Orchard", "Root"],
            "nearest first, so the stack recedes toward the root"
        );
        assert_eq!(trail[0].role, Role::Parent { depth: 1 });
        assert_eq!(trail[1].role, Role::Parent { depth: 0 });
    }

    #[test]
    fn a_trail_mote_climbs_to_its_own_level() {
        let mut tree = tree();
        tree.select(1);
        assert_eq!(tree.depth(), 1);
        tree.ascend_to(0);
        assert_eq!(tree.depth(), 0);
        assert_eq!(labels(&tree), vec!["Home", "Places", "Lens"]);
    }

    #[test]
    fn a_model_hears_about_the_leaf_that_fired() {
        #[derive(Default)]
        struct Recorder {
            fired: Vec<Vec<usize>>,
        }

        impl Model for Recorder {
            fn root(&self) -> MoteSpec {
                MoteSpec {
                    role:        Role::Group {
                        children: 1,
                        groups:   0,
                    },
                    label:       SmolStr::new_static("Root"),
                    description: None,
                }
            }

            fn children(&self, path: &[usize]) -> Vec<MoteSpec> {
                if path.is_empty() {
                    vec![MoteSpec {
                        role:        Role::Action,
                        label:       SmolStr::new_static("Fire"),
                        description: None,
                    }]
                } else {
                    Vec::new()
                }
            }

            fn activate(&mut self, path: &[usize]) {
                self.fired.push(path.to_vec());
            }
        }

        let mut tree = Tree::new(Recorder::default());
        assert_eq!(tree.select(0), Navigation::Activated("Fire".into()));
        assert_eq!(tree.model().fired, vec![vec![0]]);
        assert_eq!(tree.depth(), 0, "activating does not move");
    }

    #[test]
    fn a_lazy_model_is_only_asked_for_the_level_that_is_open() {
        use std::cell::RefCell;

        struct Lazy {
            asked: RefCell<Vec<Vec<usize>>>,
        }

        impl Model for Lazy {
            fn root(&self) -> MoteSpec {
                MoteSpec {
                    role:        Role::Group {
                        children: 2,
                        groups:   2,
                    },
                    label:       SmolStr::new_static("Root"),
                    description: None,
                }
            }

            fn children(&self, path: &[usize]) -> Vec<MoteSpec> {
                self.asked.borrow_mut().push(path.to_vec());
                if path.len() > 2 {
                    return Vec::new();
                }
                (0..2)
                    .map(|index| MoteSpec {
                        role:        Role::Group {
                            children: 2,
                            groups:   2,
                        },
                        label:       SmolStr::new(format!("{}-{index}", path.len())),
                        description: None,
                    })
                    .collect()
            }

            fn activate(&mut self, _path: &[usize]) {}
        }

        let tree = Tree::new(Lazy {
            asked: RefCell::new(Vec::new()),
        });
        drop(tree.level());
        assert_eq!(
            tree.model().asked.borrow().as_slice(),
            [Vec::new()],
            "an unbounded library costs nothing until it is opened"
        );
    }
}
