//! The mote graph: what a script builds and edits, and the level a surface
//! currently has open into it.

use std::{
    cell::{
        Cell,
        RefCell,
    },
    rc::{
        Rc,
        Weak,
    },
};

use smol_str::SmolStr;
use wired_guest::math::{
    Color,
    Vec3,
};

use crate::{
    fit::{
        self,
        Fit,
    },
    glyph::Glyph,
    mote::{
        Arrange,
        MoteSpec,
        Role,
    },
    wired::scene::document::Document,
};

/// A prim in the document VUI draws into, naming its place in the scene
/// rather than any behaviour of its own.
pub type PrimId = (u64, u64);

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

    /// Whether choosing this leaves a thing behind rather than doing
    /// something. Both buttons deliver it; the grip only chooses where.
    #[must_use]
    pub const fn delivers(self) -> bool {
        matches!(self, Self::Item)
    }
}

/// Something a mote told the viewer, and the surface-clock time it was said at.
struct Notice {
    message: SmolStr,
    posted:  f64,
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
    /// What the mote is, drawn inside its shell: a prim in the document VUI
    /// draws into. A plain id rather than a handle, so holding one costs
    /// nothing and a surface can keep drawing it after the consumer's own
    /// handle is gone.
    icon:        Option<PrimId>,
    /// The icon's measured fit, cleared whenever `icon` is set. Measuring
    /// reads every piece's stream back over the interface, which is cheap
    /// once and not something to repeat every frame.
    fit:         Cell<Option<Fit>>,
    /// The dot grid drawn on its face in place of the icon. Shared so that a
    /// frame's spec copies a pointer rather than the grid.
    glyph:       Option<Rc<Glyph>>,
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
    /// What the mote last told the viewer, and when.
    notice:      Option<Notice>,
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
            fit: Cell::new(None),
            glyph: None,
            unique: false,
            arrange: Arrange::Orbit,
            tint: None,
            film: crate::palette::FILM,
            frost: 0.0,
            notice: None,
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

    /// A number identifying this mote, stable for as long as some handle
    /// keeps it alive. A consumer can key a map by this instead of scanning
    /// handles with [`Mote::is`].
    #[must_use]
    pub fn id(&self) -> u64 {
        Rc::as_ptr(&self.0) as u64
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

    #[must_use]
    pub fn description(&self) -> Option<SmolStr> {
        self.0.borrow().description.clone()
    }

    pub fn set_icon(&self, icon: Option<PrimId>) {
        let mut data = self.0.borrow_mut();
        data.icon = icon;
        data.fit.set(None);
    }

    /// The fit this mote's icon wears, measured once and remembered until
    /// [`Mote::set_icon`] changes it.
    pub fn icon_fit(&self, doc: &Document, fraction: f32) -> anyhow::Result<Fit> {
        if let Some(fit) = self.0.borrow().fit.get() {
            return Ok(fit);
        }
        let Some(icon) = self.icon() else {
            return Ok(fit::fit(Vec3::ZERO, Vec3::ZERO, fraction));
        };
        let (min, max) = fit::measure(doc, icon)?;
        let computed = fit::fit(min, max, fraction);
        self.0.borrow().fit.set(Some(computed));
        Ok(computed)
    }

    /// Draws `glyph` on the face instead of the icon, or the icon again on
    /// none.
    pub fn set_glyph(&self, glyph: Option<Glyph>) {
        self.0.borrow_mut().glyph = glyph.map(Rc::new);
    }

    pub fn set_unique(&self, unique: bool) {
        self.0.borrow_mut().unique = unique;
    }

    #[must_use]
    pub fn is_unique(&self) -> bool {
        self.0.borrow().unique
    }

    #[must_use]
    pub fn icon(&self) -> Option<PrimId> {
        self.0.borrow().icon
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

    pub fn set_tint(&self, tint: Option<Color>) {
        self.0.borrow_mut().tint = tint;
    }

    pub fn set_film(&self, film: f32) {
        self.0.borrow_mut().film = film;
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

    /// Tells the viewer `message`, at `now` on the surface clock. A later
    /// notice replaces an earlier one.
    pub fn notify(&self, message: &str, now: f64) {
        self.0.borrow_mut().notice = Some(Notice {
            message: SmolStr::new(message),
            posted:  now,
        });
    }

    /// The notice still showing at `now`, if it was posted less than
    /// `duration` seconds ago.
    #[must_use]
    pub fn notice(&self, now: f64, duration: f32) -> Option<SmolStr> {
        let data = self.0.borrow();
        let notice = data.notice.as_ref()?;
        let age = now - notice.posted;
        (0.0..f64::from(duration))
            .contains(&age)
            .then(|| notice.message.clone())
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
            icon:        data.icon.is_some() || data.glyph.is_some(),
            glyph:       data.glyph.clone(),
            tint:        data.tint,
            film:        data.film,
            frost:       data.frost,
            // A notice has a clock, which only a surface keeps: it fills this
            // in when it draws.
            notice:      None,
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

/// Fills in the notice each of `specs` is showing at `now`, for `motes` drawn
/// slot for slot.
///
/// The way back is a surface's own mote standing in for the level it leaves,
/// so a notice posted to that level is not shown on it.
pub fn noticed(motes: &[Mote], specs: &mut [MoteSpec], now: f64, duration: f32) {
    for (spec, mote) in specs.iter_mut().zip(motes) {
        if !matches!(spec.role, Role::Parent { .. }) {
            spec.notice = mote.notice(now, duration);
        }
    }
}

/// The mark the top of the tree wears when it has no glyph of its own: a
/// diamond round a single dot, like a place on a map. The root is not a mote
/// anyone draws, so it has no face to borrow, and a bead with none would read
/// as a hole in the row.
///
/// ```text
/// ..#..
/// .#.#.
/// #.#.#
/// .#.#.
/// ..#..
/// ```
const TOP_MARK: [u8; 4] = [0x44, 0x55, 0x45, 0x00];
const TOP_MARK_SIZE: u8 = 5;

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
    /// The mark the top wears, built once rather than every frame.
    top:  Option<Rc<Glyph>>,
    /// For each level, the name it was last phrased from and the phrase, so
    /// a trail drawn every frame formats a name only when it changes.
    ups:  Vec<(SmolStr, SmolStr)>,
}

impl Tree {
    #[must_use]
    pub fn new(root: Mote) -> Self {
        Self {
            root,
            path: Vec::new(),
            top: Glyph::new(TOP_MARK_SIZE, &TOP_MARK).ok().map(Rc::new),
            ups: Vec::new(),
        }
    }

    /// What the bead for the level `level` deep, which is `mote`'s, draws as.
    ///
    /// It keeps the level's hue and glyph and nothing else of it: not its
    /// icon prim, so a bead can never hold on to a prim that belongs to the
    /// level, and not its name, which the placard says instead as the way up.
    fn bead(&mut self, level: usize, mote: &Mote) -> MoteSpec {
        let spec = mote.spec();
        let glyph = spec
            .glyph
            .or_else(|| if level == 0 { self.top.clone() } else { None });
        MoteSpec {
            role: Role::Ancestor { level },
            label: self.phrase(level, spec.label),
            description: None,
            active: false,
            icon: glyph.is_some(),
            glyph,
            tint: spec.tint,
            film: spec.film,
            frost: spec.frost,
            notice: None,
        }
    }

    /// The way up to the level `level` deep, named `label`.
    fn phrase(&mut self, level: usize, label: SmolStr) -> SmolStr {
        if level == 0 {
            return SmolStr::new_static("Up to the top");
        }
        if self.ups.len() <= level {
            self.ups
                .resize(level + 1, (SmolStr::default(), SmolStr::default()));
        }
        let entry = &mut self.ups[level];
        if entry.1.is_empty() || entry.0 != label {
            entry.1 = SmolStr::new(format!("Up to {label}"));
            entry.0 = label;
        }
        entry.1.clone()
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
            role:        Role::Parent { depth },
            label:       SmolStr::new_static("Back"),
            description: Some(SmolStr::new(format!("Back to {}.", self.outer().label()))),
            active:      false,
            icon:        false,
            glyph:       None,
            tint:        None,
            film:        crate::palette::FILM,
            frost:       0.0,
            notice:      None,
        }
    }

    /// The ancestor beads to draw: one for every level strictly above the one
    /// the way back returns to, root first, since that level is already one
    /// press away. At most `limit` of them, the nearest levels when there are
    /// more, each still naming the level it really is.
    pub fn trail(&mut self, limit: usize) -> (Vec<Mote>, Vec<MoteSpec>) {
        let beads = self.depth().saturating_sub(1);
        (beads.saturating_sub(limit)..beads)
            .map(|level| {
                let mote = self.level_mote(level);
                let spec = self.bead(level, &mote);
                (mote, spec)
            })
            .unzip()
    }

    /// The mote whose level is `level` deep, counting the root as 0.
    fn level_mote(&self, level: usize) -> Mote {
        level
            .checked_sub(1)
            .and_then(|index| self.path.get(index))
            .unwrap_or(&self.root)
            .clone()
    }

    /// Climbs straight out to `level`, root 0, in one step. Only a level
    /// that has a bead is climbed to: the one the way back returns to, and
    /// the open one, are the way back's business.
    pub fn ascend(&mut self, level: usize) -> Navigation {
        if level.saturating_add(1) >= self.depth() {
            return Navigation::None;
        }
        self.path.truncate(level);
        Navigation::Collapsed(self.open())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mote(kind: Kind, label: &str) -> Mote {
        Mote::new(kind, label)
    }

    fn holding(label: &str, children: Vec<Mote>) -> Mote {
        let group = mote(Kind::Group, label);
        for child in children {
            assert!(group.add_child(&child));
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
        tree.level_with_motes()
            .1
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
        assert_eq!(labels(&mut tree), vec!["Back", "Atrium", "Club"]);
        assert_eq!(tree.level_with_motes().1[0].role, Role::Parent { depth: 1 });
    }

    #[test]
    fn the_way_back_wears_none_of_the_level_it_leaves() {
        let mut tree = tree();
        tree.select(1);

        let back = &tree.level_with_motes().1[0];
        assert_eq!(back.label, "Back", "a name, not the group's");
        assert!(!back.icon, "and not the group's glyph either");
        assert_eq!(back.tint, None);
    }

    #[test]
    fn the_way_back_names_where_it_goes() {
        let citrus = holding("Citrus", vec![mote(Kind::Item, "Lemon")]);
        let mut tree = Tree::new(holding("Produce", vec![citrus]));
        tree.select(0);

        assert_eq!(
            tree.level_with_motes().1[0].description.as_deref(),
            Some("Back to Produce."),
            "the level it leaves is named where it can be read, not worn"
        );
    }

    #[test]
    fn a_tinted_level_does_not_colour_the_way_out_of_it() {
        let inner = holding("Inner", Vec::new());
        inner.set_tint(Some(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }));
        let mut tree = Tree::new(holding("Outer", vec![inner]));
        tree.select(0);

        assert_eq!(tree.level_with_motes().1[0].tint, None);
    }

    #[test]
    fn slot_zero_collapses_only_when_nested() {
        let mut tree = tree();
        assert!(matches!(tree.select(0), Navigation::Cast));
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
    fn an_item_is_the_one_kind_that_leaves_something_behind() {
        assert!(Kind::Item.delivers());
        for doing in [Kind::Action, Kind::Toggle, Kind::Cast, Kind::Group] {
            assert!(!doing.delivers(), "{doing:?} does something instead");
        }
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
    fn a_glyph_is_a_face_of_its_own_and_clearing_it_gives_the_face_back() {
        let node = mote(Kind::Group, "Tools");
        assert!(!node.spec().icon);

        let glyph = Glyph::new(3, &[0b0001_0000, 0]).expect("a three grid");
        node.set_glyph(Some(glyph));
        let spec = node.spec();
        assert!(spec.icon, "a group wearing a glyph previews no contents");
        assert!(spec.glyph.is_some());

        node.set_glyph(None);
        assert!(node.spec().glyph.is_none());
        assert!(!node.spec().icon);
    }

    #[test]
    fn a_notice_shows_for_its_duration_and_then_goes() {
        let node = mote(Kind::Cast, "Home");
        assert_eq!(node.notice(0.0, 3.0), None, "nothing said yet");

        node.notify("No home space set.", 10.0);
        assert_eq!(
            node.notice(10.0, 3.0).as_deref(),
            Some("No home space set.")
        );
        assert_eq!(
            node.notice(12.9, 3.0).as_deref(),
            Some("No home space set.")
        );
        assert_eq!(node.notice(13.0, 3.0), None, "run out");
    }

    #[test]
    fn a_later_notice_replaces_an_earlier_one_and_restarts_the_clock() {
        let node = mote(Kind::Cast, "Home");
        node.notify("first", 0.0);
        node.notify("second", 2.0);
        assert_eq!(node.notice(4.0, 3.0).as_deref(), Some("second"));
    }

    #[test]
    fn the_way_back_does_not_wear_the_level_s_notice() {
        let inner = mote(Kind::Action, "inner");
        let outer = holding("outer", vec![inner.clone()]);
        outer.notify("posted to the level", 0.0);
        inner.notify("posted to the leaf", 0.0);

        let mut tree = Tree::new(holding("root", vec![outer]));
        tree.select(0);
        let (motes, mut specs) = tree.level_with_motes();
        noticed(&motes, &mut specs, 1.0, 3.0);

        assert_eq!(specs[0].role, Role::Parent { depth: 1 });
        assert_eq!(specs[0].notice, None);
        assert_eq!(specs[1].notice.as_deref(), Some("posted to the leaf"));
    }

    /// A tree `levels` deep, each level holding the next and a leaf, opened to
    /// the bottom. Level `n` is labelled `L<n>`, the root `L0`.
    fn descended(levels: usize) -> Tree {
        let mut node = mote(Kind::Action, "leaf");
        for level in (0..levels).rev() {
            let leaf = mote(Kind::Action, &format!("leaf{level}"));
            node = holding(&format!("L{level}"), vec![leaf, node]);
        }
        let mut tree = Tree::new(node);
        for _ in 1..levels {
            let slot = usize::from(tree.is_nested()) + 1;
            assert!(matches!(tree.select(slot), Navigation::Bloomed(_)));
        }
        tree
    }

    fn trail_labels(tree: &mut Tree, limit: usize) -> Vec<String> {
        tree.trail(limit)
            .1
            .iter()
            .map(|spec| spec.label.to_string())
            .collect()
    }

    #[test]
    fn beads_stand_for_every_level_above_the_one_the_way_back_returns_to() {
        for (levels, beads) in [(1, 0), (2, 0), (3, 1), (4, 2), (5, 3)] {
            let mut tree = descended(levels);
            assert_eq!(tree.depth(), levels - 1);
            let (motes, specs) = tree.trail(8);
            assert_eq!(specs.len(), beads, "depth {}", levels - 1);
            assert_eq!(motes.len(), beads);
        }
    }

    #[test]
    fn the_trail_runs_from_the_root_down_and_names_where_each_goes() {
        let mut tree = descended(5);
        assert_eq!(
            trail_labels(&mut tree, 8),
            vec!["Up to the top", "Up to L1", "Up to L2"]
        );
        let (motes, specs) = tree.trail(8);
        let levels = specs.iter().map(|spec| spec.role).collect::<Vec<_>>();
        assert_eq!(
            levels,
            vec![
                Role::Ancestor { level: 0 },
                Role::Ancestor { level: 1 },
                Role::Ancestor { level: 2 }
            ]
        );
        assert_eq!(motes[1].label(), "L1");
        assert_eq!(
            tree.level_with_motes().1[0].description.as_deref(),
            Some("Back to L3."),
            "and the way back covers the level just above"
        );
    }

    #[test]
    fn a_deep_trail_keeps_the_nearest_levels_under_their_real_numbers() {
        let mut tree = descended(8);
        let (_, specs) = tree.trail(2);
        let levels = specs.iter().map(|spec| spec.role).collect::<Vec<_>>();
        assert_eq!(
            levels,
            vec![Role::Ancestor { level: 4 }, Role::Ancestor { level: 5 }]
        );
    }

    #[test]
    fn a_bead_wears_its_level_s_glyph_and_hue_but_not_its_icon() {
        let mut tree = descended(4);
        let level = tree.level_mote(1);
        level.set_icon(Some((1, 2)));
        level.set_tint(Some(Color {
            r: 0.2,
            g: 0.4,
            b: 0.9,
            a: 1.0,
        }));
        let bare = tree.trail(8).1;
        assert!(
            bare[1].glyph.is_none() && !bare[1].icon,
            "no icon prim worn"
        );
        assert!(bare[1].tint.is_some());

        level.set_glyph(Some(Glyph::new(3, &[0b0001_0000, 0]).expect("a grid")));
        let worn = tree.trail(8).1;
        assert!(worn[1].icon && worn[1].glyph.is_some());
        assert_eq!(worn[1].description, None);
        assert_eq!(worn[1].notice, None);
    }

    #[test]
    fn a_renamed_level_is_phrased_afresh_and_an_unchanged_one_is_not() {
        let mut tree = descended(4);
        assert_eq!(trail_labels(&mut tree, 8)[1], "Up to L1");
        tree.level_mote(1).set_label("Plaza");
        assert_eq!(trail_labels(&mut tree, 8)[1], "Up to Plaza");
        assert_eq!(trail_labels(&mut tree, 8)[1], "Up to Plaza");
    }

    #[test]
    fn the_root_wears_a_mark_of_its_own_unless_it_has_a_glyph() {
        let mut tree = descended(4);
        let marked = tree.trail(8).1[0].glyph.clone().expect("a fixed mark");
        assert_eq!(marked.size(), 5);

        tree.level_mote(0)
            .set_glyph(Some(Glyph::new(3, &[0b0001_0000, 0]).expect("a grid")));
        let own = tree.trail(8).1[0].glyph.clone().expect("its own");
        assert_eq!(own.size(), 3);
    }

    #[test]
    fn ascending_pops_straight_to_the_level_in_one_step() {
        let mut tree = descended(5);
        assert_eq!(tree.depth(), 4);

        let Navigation::Collapsed(reached) = tree.ascend(1) else {
            panic!("a bead collapses");
        };
        assert_eq!(reached.label(), "L1", "the level now open");
        assert_eq!(tree.depth(), 1);
        assert!(tree.open().is(&reached));
        assert!(
            tree.trail(8).0.is_empty(),
            "and the trail is as short as the climb"
        );
    }

    #[test]
    fn ascending_to_the_root_leaves_the_tree() {
        let mut tree = descended(4);
        assert!(matches!(tree.ascend(0), Navigation::Collapsed(top) if top.label() == "L0"));
        assert_eq!(tree.depth(), 0);
    }

    #[test]
    fn only_a_level_with_a_bead_can_be_ascended_to() {
        let mut tree = descended(4);
        assert_eq!(tree.depth(), 3);
        for level in [2, 3, 99] {
            assert!(matches!(tree.ascend(level), Navigation::None), "{level}");
        }
        assert_eq!(tree.depth(), 3, "nothing moved");
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
