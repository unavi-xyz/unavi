use std::{
    cell::RefCell,
    rc::{
        Rc,
        Weak,
    },
};

use smol_str::SmolStr;
use wired_scene::types::Color;

use crate::{
    mote::{
        Arrange,
        MoteSpec,
        Role,
    },
    wired::scene::types::Prim,
};

/// What a mote is: how it draws, and what selecting it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Fires and is done — a pulse.
    Action,
    /// Turns on, and off again when it is next chosen.
    Toggle,
    /// Can be pulled out and left in the room, or filed into a grid.
    Item,
    /// Opens a cast site, which fills while it is held.
    Cast,
    /// Opens into its children, in whatever shape it is set to arrange them.
    Group,
}

impl Kind {
    #[must_use]
    pub const fn holds_children(self) -> bool {
        matches!(self, Self::Group)
    }
}

/// A mote, and whatever it holds.
///
/// Shared rather than owned: a handle kept after the mote is mounted still
/// names the mote a surface is drawing, so a level edited while it is up
/// redraws. One parent at most — adding it somewhere else moves it.
#[derive(Clone)]
pub struct Mote(Rc<RefCell<Data>>);

struct Data {
    kind:        Kind,
    label:       SmolStr,
    description: Option<SmolStr>,
    /// What the mote is, drawn inside its shell. Owned rather than borrowed
    /// so a surface can keep drawing it after the consumer's handle is gone.
    icon:        Option<Prim>,
    /// Whether this mote stands for the one of its thing rather than for a
    /// source of them. Meaningless on anything but an item.
    unique:      bool,
    /// The shape this mote's own level takes. Meaningless on anything that
    /// holds no children.
    arrange:     Arrange,
    /// The hue its glass carries. Identity, where every other colour a
    /// surface uses is state.
    tint:        Option<Color>,
    /// How much iridescent film the shell wears, `0` none to `1` the full
    /// bubble. Subtle by default so the hue stays legible.
    film:        f32,
    /// How much the shell's rim is frosted, `0` for clear glass.
    frost:       f32,
    /// Whether the mote is on: a toggle, not a pulse.
    active:      bool,
    parent:      Weak<RefCell<Self>>,
    children:    Vec<Mote>,
}

impl Mote {
    #[must_use]
    pub fn new(kind: Kind, label: &str) -> Self {
        Self(Rc::new(RefCell::new(Data {
            kind,
            label: SmolStr::new(label),
            description: None,
            icon: None,
            unique: false,
            arrange: Arrange::Orbit,
            tint: None,
            film: crate::palette::FILM,
            frost: 0.0,
            active: false,
            parent: Weak::new(),
            children: Vec::new(),
        })))
    }

    /// Whether both handles name the same mote.
    #[must_use]
    pub fn is(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    #[must_use]
    pub fn kind(&self) -> Kind {
        self.0.borrow().kind
    }

    #[must_use]
    pub fn label(&self) -> SmolStr {
        self.0.borrow().label.clone()
    }

    pub fn set_label(&self, label: &str) {
        self.0.borrow_mut().label = SmolStr::new(label);
    }

    pub fn describe(&self, text: &str) {
        self.0.borrow_mut().description = Some(SmolStr::new(text));
    }

    pub fn set_icon(&self, icon: Option<Prim>) {
        self.0.borrow_mut().icon = icon;
    }

    pub fn set_unique(&self, unique: bool) {
        self.0.borrow_mut().unique = unique;
    }

    #[must_use]
    pub fn is_unique(&self) -> bool {
        self.0.borrow().unique
    }

    /// Reads the icon in place: a handle is only cloned by a host call, so
    /// nothing borrows one per frame.
    pub fn with_icon<T>(&self, read: impl FnOnce(&Prim) -> T) -> Option<T> {
        self.0.borrow().icon.as_ref().map(read)
    }

    #[must_use]
    pub fn has_icon(&self) -> bool {
        self.0.borrow().icon.is_some()
    }

    /// Takes `child` from whatever it was under and puts it here.
    ///
    /// Refused when `child` is already an ancestor, which is the only way a
    /// level could come to contain itself.
    #[must_use]
    pub fn add_child(&self, child: &Self) -> bool {
        if child.holds(self) {
            return false;
        }
        child.orphan();
        child.0.borrow_mut().parent = Rc::downgrade(&self.0);
        self.0.borrow_mut().children.push(child.clone());
        true
    }

    /// Takes `child` out of this level. Any handle to it stays good, so it can
    /// be put somewhere else.
    pub fn remove_child(&self, child: &Self) {
        if !self.holds_child(child) {
            return;
        }
        child.orphan();
    }

    pub fn clear(&self) {
        let children = std::mem::take(&mut self.0.borrow_mut().children);
        for child in children {
            child.0.borrow_mut().parent = Weak::new();
        }
    }

    /// Takes this mote out of whatever level it is under, wherever that is.
    fn orphan(&self) {
        let parent = {
            let mut data = self.0.borrow_mut();
            std::mem::take(&mut data.parent).upgrade()
        };
        let Some(parent) = parent else {
            return;
        };
        parent
            .borrow_mut()
            .children
            .retain(|child| !Rc::ptr_eq(&child.0, &self.0));
    }

    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        self.0.borrow().parent.upgrade().map(Self)
    }

    #[must_use]
    pub fn children(&self) -> Vec<Self> {
        self.0.borrow().children.clone()
    }

    /// How this mote's own level arranges when it opens.
    #[must_use]
    pub fn arrange(&self) -> Arrange {
        self.0.borrow().arrange
    }

    pub fn set_arrange(&self, arrange: Arrange) {
        self.0.borrow_mut().arrange = arrange;
    }

    #[must_use]
    pub fn tint(&self) -> Option<Color> {
        self.0.borrow().tint
    }

    pub fn set_tint(&self, tint: Option<Color>) {
        self.0.borrow_mut().tint = tint;
    }

    #[must_use]
    pub fn film(&self) -> f32 {
        self.0.borrow().film
    }

    pub fn set_film(&self, film: f32) {
        self.0.borrow_mut().film = film;
    }

    #[must_use]
    pub fn frost(&self) -> f32 {
        self.0.borrow().frost
    }

    pub fn set_frost(&self, frost: f32) {
        self.0.borrow_mut().frost = frost;
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.0.borrow().active
    }

    pub fn set_active(&self, active: bool) {
        self.0.borrow_mut().active = active;
    }

    #[must_use]
    pub fn spec(&self) -> MoteSpec {
        let data = self.0.borrow();
        MoteSpec {
            role:        match data.kind {
                Kind::Action => Role::Action,
                Kind::Toggle => Role::Toggle,
                Kind::Item => Role::Item {
                    unique: data.unique,
                },
                Kind::Cast => Role::Cast,
                Kind::Group => Role::Group {
                    children: data.children.len(),
                    groups:   data
                        .children
                        .iter()
                        .filter(|child| child.kind().holds_children())
                        .count(),
                    arrange:  data.arrange,
                },
            },
            label:       data.label.clone(),
            description: data.description.clone(),
            active:      data.active,
            icon:        data.icon.is_some(),
            tint:        data.tint,
            film:        data.film,
            frost:       data.frost,
        }
    }

    fn holds_child(&self, other: &Self) -> bool {
        self.0.borrow().children.iter().any(|child| child.is(other))
    }

    fn holds(&self, other: &Self) -> bool {
        self.is(other)
            || self
                .0
                .borrow()
                .children
                .iter()
                .any(|child| child.holds(other))
    }
}

/// What a selection did, for a surface to report.
pub enum Navigation {
    /// The way back was taken; the mote is the level now open.
    Collapsed(Mote),
    /// A container opened; the mote is the level now open.
    Bloomed(Mote),
    /// A leaf fired.
    Activated(Mote),
    /// A consequential mote; a cast site should open on it.
    Cast(Mote),
    None,
}

/// The levels open into a tree of motes, and so the level currently drawn.
///
/// The path is the motes themselves, not their positions: a level detached
/// while it is open is climbed out of, rather than the path quietly coming to
/// mean whatever moved into that slot.
///
/// Slot 0 is the way back whenever the path is non-empty; nothing else is ever
/// placed there.
pub struct Tree {
    root: Mote,
    path: Vec<Mote>,
}

impl Tree {
    #[must_use]
    pub const fn new(root: Mote) -> Self {
        Self {
            root,
            path: Vec::new(),
        }
    }

    /// Climbs out of every level that no longer hangs where it was opened
    /// from.
    fn prune(&mut self) {
        let mut level = self.root.clone();
        let mut kept = 0;
        for mote in &self.path {
            if !level.holds_child(mote) {
                break;
            }
            level = mote.clone();
            kept += 1;
        }
        self.path.truncate(kept);
    }

    pub fn depth(&mut self) -> usize {
        self.prune();
        self.path.len()
    }

    pub fn is_nested(&mut self) -> bool {
        self.depth() > 0
    }

    /// The mote whose level is open.
    pub fn open(&mut self) -> Mote {
        self.prune();
        self.path.last().unwrap_or(&self.root).clone()
    }

    /// The motes at the current level, the way back first when nested.
    pub fn level_motes(&mut self) -> Vec<Mote> {
        let open = self.open();
        let children = open.children();
        if self.depth() == 0 {
            return children;
        }
        std::iter::once(open).chain(children).collect()
    }

    /// What [`Tree::level_motes`] draws as, slot for slot. The way back wears
    /// the level's own name under a role of its own.
    pub fn level(&mut self) -> Vec<MoteSpec> {
        let depth = self.depth();
        let motes = self.level_motes();
        motes
            .iter()
            .enumerate()
            .map(|(slot, mote)| {
                let spec = mote.spec();
                if depth > 0 && slot == 0 {
                    return MoteSpec {
                        role: Role::Parent { depth },
                        description: Some(SmolStr::new_static("Go back out.")),
                        // The way back is not a switch, whatever the mote it
                        // stands for happens to be.
                        active: false,
                        ..spec
                    };
                }
                spec
            })
            .collect()
    }

    /// The mote drawn at `index` of [`Tree::level`], which is the way back
    /// itself at slot 0 of a nested level.
    pub fn at_level(&mut self, index: usize) -> Option<Mote> {
        let open = self.open();
        let Some(index) = index.checked_sub(usize::from(self.is_nested())) else {
            return Some(open);
        };
        open.0.borrow().children.get(index).cloned()
    }

    pub fn arrange(&mut self) -> Arrange {
        self.open().arrange()
    }

    pub fn select(&mut self, index: usize) -> Navigation {
        if self.is_nested() && index == 0 {
            self.path.pop();
            return Navigation::Collapsed(self.open());
        }
        let Some(child) = self.at_level(index) else {
            return Navigation::None;
        };
        match child.kind() {
            Kind::Group => {
                self.path.push(child.clone());
                Navigation::Bloomed(child)
            }
            Kind::Cast => Navigation::Cast(child),
            // A toggle carries its own state, so choosing it flips it before
            // anyone is told: a consumer reads back the state it is in now
            // rather than being handed an edge to keep track of.
            Kind::Toggle => {
                let on = child.is_active();
                child.set_active(!on);
                Navigation::Activated(child)
            }
            Kind::Action | Kind::Item => Navigation::Activated(child),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mote(kind: Kind, label: &str) -> Mote {
        Mote::new(kind, label)
    }

    fn holding(label: &str, children: Vec<Mote>) -> Mote {
        let group = mote(Kind::Group, label);
        for child in &children {
            assert!(group.add_child(child));
        }
        group
    }

    fn tree() -> Tree {
        Tree::new(holding(
            "Root",
            vec![
                mote(Kind::Cast, "Home"),
                holding(
                    "Places",
                    vec![mote(Kind::Action, "Atrium"), mote(Kind::Action, "Club")],
                ),
                mote(Kind::Action, "Lens"),
            ],
        ))
    }

    fn labels(tree: &mut Tree) -> Vec<String> {
        tree.level()
            .into_iter()
            .map(|spec| spec.label.to_string())
            .collect()
    }

    #[test]
    fn the_root_level_has_no_parent_bead() {
        let mut tree = tree();
        assert_eq!(tree.depth(), 0);
        assert_eq!(labels(&mut tree), vec!["Home", "Places", "Lens"]);
    }

    #[test]
    fn blooming_puts_the_parent_at_slot_zero() {
        let mut tree = tree();
        assert!(matches!(tree.select(1), Navigation::Bloomed(mote) if mote.label() == "Places"));
        assert_eq!(labels(&mut tree), vec!["Places", "Atrium", "Club"]);
        assert_eq!(tree.level()[0].role, Role::Parent { depth: 1 });
    }

    #[test]
    fn slot_zero_collapses_only_when_nested() {
        let mut tree = tree();
        assert!(matches!(tree.select(0), Navigation::Cast(mote) if mote.label() == "Home"));
        assert_eq!(
            tree.depth(),
            0,
            "slot 0 at the root is a child, not a way back"
        );

        tree.select(1);
        assert!(matches!(tree.select(0), Navigation::Collapsed(_)));
        assert_eq!(tree.depth(), 0);
        assert_eq!(labels(&mut tree), vec!["Home", "Places", "Lens"]);
    }

    #[test]
    fn leaves_activate_without_moving() {
        let mut tree = tree();
        assert!(matches!(tree.select(2), Navigation::Activated(mote) if mote.label() == "Lens"));
        assert_eq!(tree.depth(), 0);
    }

    #[test]
    fn a_branch_counts_how_many_children_are_themselves_containers() {
        let node = holding(
            "mixed",
            vec![
                holding("a", vec![mote(Kind::Action, "x")]),
                mote(Kind::Action, "y"),
                holding("b", Vec::new()),
            ],
        );
        assert_eq!(
            node.spec().role,
            Role::Group {
                children: 3,
                groups:   2,
                arrange:  Arrange::Orbit,
            }
        );
    }

    #[test]
    fn selecting_out_of_range_does_nothing() {
        let mut tree = tree();
        assert!(matches!(tree.select(99), Navigation::None));
        assert_eq!(tree.depth(), 0);
    }

    #[test]
    fn depth_is_unbounded() {
        let mut node = mote(Kind::Action, "bottom");
        for level in 0..32 {
            node = holding(&format!("level{level}"), vec![node]);
        }
        let mut tree = Tree::new(node);
        let mut bloomed = 0;
        loop {
            let slot = usize::from(tree.is_nested());
            if !matches!(tree.select(slot), Navigation::Bloomed(_)) {
                break;
            }
            bloomed += 1;
        }
        // 32 nested branches, so 31 of them can be descended into before the
        // leaf at the bottom activates instead.
        assert_eq!(bloomed, 31);
        assert_eq!(tree.depth(), 31);
    }

    #[test]
    fn how_a_level_opens_is_the_group_s_own_setting() {
        let market = mote(Kind::Group, "Market");
        assert_eq!(market.arrange(), Arrange::Orbit, "an orbit unless told");
        market.set_arrange(Arrange::Grid);
        assert_eq!(market.arrange(), Arrange::Grid);
        assert_eq!(
            market.spec().role,
            Role::Group {
                children: 0,
                groups:   0,
                arrange:  Arrange::Grid,
            },
            "how it opens is the group's own setting, not a second kind of mote"
        );
    }

    #[test]
    fn a_mote_is_a_child_of_one_level_at_a_time() {
        let lemon = mote(Kind::Item, "Lemon");
        let citrus = holding("Citrus", vec![lemon.clone()]);
        let pocket = holding("Pocket", Vec::new());

        assert!(pocket.add_child(&lemon));
        assert!(
            citrus.children().is_empty(),
            "adding moves, it never copies"
        );
        assert!(pocket.children()[0].is(&lemon));
        assert!(pocket.parent().is_none());
        assert!(lemon.parent().expect("a parent").is(&pocket));
    }

    #[test]
    fn a_removed_mote_leaves_the_level_it_was_under() {
        let lemon = mote(Kind::Item, "Lemon");
        let citrus = holding("Citrus", vec![lemon.clone(), mote(Kind::Item, "Lime")]);

        citrus.remove_child(&lemon);
        assert_eq!(citrus.children().len(), 1);
        assert!(lemon.parent().is_none());
        assert_eq!(lemon.label(), "Lemon", "the handle stays good");
    }

    #[test]
    fn removing_a_mote_from_a_level_it_is_not_under_does_nothing() {
        let lemon = mote(Kind::Item, "Lemon");
        let citrus = holding("Citrus", vec![lemon.clone()]);
        let pocket = holding("Pocket", Vec::new());

        pocket.remove_child(&lemon);
        assert_eq!(citrus.children().len(), 1, "still where it was");
        assert!(lemon.parent().expect("a parent").is(&citrus));
    }

    #[test]
    fn clearing_a_level_lets_its_motes_be_put_elsewhere() {
        let lemon = mote(Kind::Item, "Lemon");
        let citrus = holding("Citrus", vec![lemon.clone()]);
        let pocket = holding("Pocket", Vec::new());

        citrus.clear();
        assert!(citrus.children().is_empty());
        assert!(pocket.add_child(&lemon));
        assert!(pocket.children()[0].is(&lemon));
    }

    #[test]
    fn a_level_cannot_be_made_to_contain_itself() {
        let citrus = holding("Citrus", vec![mote(Kind::Item, "Lemon")]);
        let produce = holding("Produce", vec![citrus.clone()]);

        assert!(
            !citrus.add_child(&produce),
            "an ancestor cannot be made a child"
        );
        assert!(!citrus.add_child(&citrus));
        assert_eq!(citrus.children().len(), 1);
        assert_eq!(produce.children().len(), 1);
    }

    #[test]
    fn a_label_written_after_mounting_is_what_the_level_draws() {
        let lemon = mote(Kind::Item, "Lemon");
        let mut tree = Tree::new(holding("Citrus", vec![lemon.clone()]));

        lemon.set_label("Lime");
        assert_eq!(labels(&mut tree), vec!["Lime"]);
    }

    #[test]
    fn removing_the_level_that_is_open_climbs_out_of_it() {
        let citrus = holding("Citrus", vec![mote(Kind::Item, "Lemon")]);
        let produce = holding("Produce", vec![citrus.clone(), mote(Kind::Item, "Pear")]);
        let mut tree = Tree::new(produce.clone());

        tree.select(0);
        assert_eq!(tree.depth(), 1);

        produce.remove_child(&citrus);
        assert_eq!(tree.depth(), 0, "the level it was inside is gone");
        assert_eq!(labels(&mut tree), vec!["Pear"]);
    }
}
