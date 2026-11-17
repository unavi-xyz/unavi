//! The levels open into a tree of motes, and what selecting one does.

use smol_str::SmolStr;

use super::{
    Kind,
    Mote,
};
use crate::{
    part::spec::{
        Arrange,
        MoteSpec,
        Role,
    },
    tokens::state::Condition,
};

/// What a selection did, for a surface to report.
pub enum Navigation {
    /// The way back was taken; the mote is the level now open.
    Collapsed(Mote),
    /// A container opened; the mote is the level now open.
    Bloomed(Mote),
    /// A leaf fired.
    Activated(Mote),
    /// A consequential mote; nothing to report, since its cast site opens on
    /// press rather than on this selection.
    Cast,
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

    /// [`Tree::level_motes`] and what each draws as, slot for slot, in one
    /// pass.
    pub fn level_with_motes(&mut self) -> (Vec<Mote>, Vec<MoteSpec>) {
        let depth = self.depth();
        let back = (depth > 0).then(|| self.back(depth));
        let motes = self.level_motes();
        let specs = motes
            .iter()
            .enumerate()
            .map(|(slot, mote)| match (&back, slot) {
                (Some(back), 0) => back.clone(),
                _ => mote.spec(),
            })
            .collect();
        (motes, specs)
    }

    /// The way back, which is VUI's own mote rather than the level's: it
    /// wears none of the group's identity, so it cannot be read as another
    /// way in to it.
    fn back(&mut self, depth: usize) -> MoteSpec {
        MoteSpec {
            role:             Role::Parent { depth },
            label:            SmolStr::new_static("Back"),
            description:      Some(SmolStr::new(format!("Back to {}.", self.outer().label()))),
            description_icon: None,
            condition:        Condition::default(),
            icon:             false,
            glyph:            None,
            tint:             None,
            film:             crate::tokens::palette::FILM,
            frost:            0.0,
            notice:           None,
        }
    }

    /// The level the way back returns to, which is the one above the open one.
    fn outer(&mut self) -> Mote {
        self.prune();
        self.path
            .len()
            .checked_sub(2)
            .and_then(|index| self.path.get(index))
            .unwrap_or(&self.root)
            .clone()
    }

    /// The mote drawn at slot `index` of the current level, which is the way
    /// back itself at slot 0 of a nested level.
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
            Kind::Cast => Navigation::Cast,
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
