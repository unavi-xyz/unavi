use std::{
    collections::{
        BTreeMap,
        BTreeSet,
        HashMap,
        HashSet,
    },
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};

use smol_str::SmolStr;
use thiserror::Error;

use crate::{
    attributes::{
        Attribute,
        slots::is_slot_name,
    },
    id::PrimId,
    key,
    meta::DocMeta,
    property::{
        Parent,
        Property,
        PropertyError,
    },
    state::{
        entry::{
            Entry,
            Stamp,
        },
        event::SceneEvent,
        prim::{
            Origin,
            PrimState,
        },
    },
};

pub mod entry;
pub mod event;
pub mod prim;
pub mod save;

#[cfg(test)] mod tests;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("unknown prim {0}")]
    UnknownPrim(PrimId),
    #[error("invalid property name {0:?}")]
    Name(String),
    #[error("property {0}")]
    Property(#[from] PropertyError),
    #[error("postcard {0}")]
    Postcard(#[from] postcard::Error),
}

/// Deepest parent chain a prim may sit under.
///
/// A document nesting past this holds its deeper prims rather than realizing
/// them: resolving one prim's placement walks its whole chain, and the ECS
/// hierarchy it becomes is walked recursively again on every propagation and
/// despawn.
pub const MAX_PRIM_DEPTH: usize = 512;

/// Most prims one document may realize at once.
///
/// Enforced here rather than at any one consumer: entries arrive from peers
/// over document sync, which never passes through the authoring API where the
/// per-document `Stock::Prims` quota is charged. Prims past the cap stay held,
/// exactly as an orphan does, and realize if room frees up.
pub const MAX_REALIZED_PRIMS: usize = 100_000;

/// Where a prim sits once the tree's integrity rules have been applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    Root,
    Child(PrimId),
    /// Held: the parent chain reaches a prim that does not exist.
    Unrealized,
}

/// The live scene.
///
/// A flat map of prims and their properties, with the same shape as the entry
/// set, so saving is a per-key diff rather than a whole-document snapshot.
#[derive(Debug)]
pub struct SceneState {
    meta:     DocMeta,
    prims:    HashMap<PrimId, PrimState>,
    /// Parent id to children, including parents that do not exist yet, which
    /// is what lets an orphan be picked up when its parent arrives.
    children: HashMap<PrimId, BTreeSet<PrimId>>,
    /// Realized prims and their effective parent, `None` for a document root.
    realized: HashMap<PrimId, Option<PrimId>>,
    events:   Vec<SceneEvent>,
}

impl Default for SceneState {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            meta:     DocMeta::default(),
            prims:    HashMap::new(),
            children: HashMap::new(),
            realized: HashMap::new(),
            events:   Vec::new(),
        }
    }

    #[must_use]
    pub const fn meta(&self) -> DocMeta {
        self.meta
    }

    #[must_use]
    pub fn get(&self, prim: PrimId) -> Option<&PrimState> {
        self.prims.get(&prim)
    }

    /// Whether a prim exists: its `parent/` property resolves to a live entry.
    /// Existence is not realization — an existing prim may still be held.
    #[must_use]
    pub fn exists(&self, prim: PrimId) -> bool {
        self.prims.get(&prim).is_some_and(|s| s.parent.is_some())
    }

    #[must_use]
    pub fn is_realized(&self, prim: PrimId) -> bool {
        self.realized.contains_key(&prim)
    }

    /// The effective parent of a realized prim, `None` if it is a document
    /// root or is not realized.
    #[must_use]
    pub fn parent(&self, prim: PrimId) -> Option<PrimId> {
        self.realized.get(&prim).copied().flatten()
    }

    pub fn prims(&self) -> impl Iterator<Item = PrimId> {
        self.realized.keys().copied()
    }

    #[must_use]
    pub fn roots(&self) -> Vec<PrimId> {
        let mut out = self
            .realized
            .iter()
            .filter_map(|(prim, parent)| parent.is_none().then_some(*prim))
            .collect::<Vec<_>>();
        out.sort_unstable();
        out
    }

    #[must_use]
    pub fn children(&self, prim: PrimId) -> Vec<PrimId> {
        let mut out = self
            .children
            .get(&prim)
            .into_iter()
            .flatten()
            .copied()
            .filter(|child| self.realized.get(child) == Some(&Some(prim)))
            .collect::<Vec<_>>();
        out.sort_unstable();
        out
    }

    pub fn drain_events(&mut self) -> Vec<SceneEvent> {
        std::mem::take(&mut self.events)
    }

    /// Replaces pending events with a full description of the realized scene.
    ///
    /// A consumer attaching to a state that was built before it existed gets
    /// the whole scene this way, rather than needing a separate initial-state
    /// path. Parents are emitted before their children so a consumer building
    /// a hierarchy never has to defer.
    pub fn resync(&mut self) {
        self.events.clear();

        let mut stack = self.roots();
        stack.reverse();
        while let Some(prim) = stack.pop() {
            self.events.push(SceneEvent::Realized {
                prim,
                parent: self.parent(prim),
            });
            self.emit_contents(prim);
            let mut children = self.children(prim);
            children.reverse();
            stack.extend(children);
        }
    }

    #[must_use]
    pub fn attribute<A: Attribute>(&self, prim: PrimId) -> Option<Result<A, postcard::Error>> {
        let payload = self.get(prim)?.property(A::KEY)?.as_attribute()?;
        Some(A::decode(payload))
    }

    #[must_use]
    pub fn relationship(&self, prim: PrimId, name: &str) -> Option<PrimId> {
        self.get(prim)?.property(name)?.as_relationship()
    }
}

/// Local writes: applied immediately, needing no capability and no storage.
/// Whether they reach other peers is the space protocol's business, and
/// whether they reach the document is an explicit save.
impl SceneState {
    pub fn create_prim(&mut self, parent: Option<PrimId>) -> PrimId {
        let prim = PrimId::new();
        self.write_parent(
            prim,
            Some(parent.map_or(Parent::Root, Parent::Prim)),
            Origin::Script,
            None,
        );
        prim
    }

    /// Inserts a prim whose id is fixed by the document or a compiled prefab,
    /// so it is byte-identical on every peer.
    pub fn insert_prim(&mut self, prim: PrimId, parent: Parent) {
        self.write_parent(prim, Some(parent), Origin::Document, None);
    }

    pub fn set_parent(&mut self, prim: PrimId, parent: Parent) -> Result<(), StateError> {
        if !self.exists(prim) {
            return Err(StateError::UnknownPrim(prim));
        }
        self.write_parent(prim, Some(parent), Origin::Script, None);
        Ok(())
    }

    pub fn remove_prim(&mut self, prim: PrimId) {
        if self.prims.contains_key(&prim) {
            self.write_parent(prim, None, Origin::Script, None);
            self.prims.remove(&prim);
        }
    }

    pub fn set_property(
        &mut self,
        prim: PrimId,
        name: &str,
        value: Property,
    ) -> Result<(), StateError> {
        if !key::is_valid_name(name) {
            return Err(StateError::Name(name.to_owned()));
        }
        let stamp = Stamp::new(now_micros(), &value.encode());
        self.write_property(prim, name, Some(value), Origin::Script, stamp);
        Ok(())
    }

    pub fn set_attribute<A: Attribute>(
        &mut self,
        prim: PrimId,
        value: &A,
    ) -> Result<(), StateError> {
        self.set_property(prim, A::KEY, Property::Attribute(value.encode()?))
    }

    pub fn set_relationship(
        &mut self,
        prim: PrimId,
        name: &str,
        target: PrimId,
    ) -> Result<(), StateError> {
        self.set_property(prim, name, Property::Relationship(target))
    }

    pub fn remove_property(&mut self, prim: PrimId, name: &str) {
        let stamp = Stamp::new(now_micros(), &[]);
        self.write_property(prim, name, None, Origin::Script, stamp);
    }

    pub fn set_slot(&mut self, prim: PrimId, name: &str, value: Vec<u8>) -> Result<(), StateError> {
        if !key::is_valid_name(name) {
            return Err(StateError::Name(name.to_owned()));
        }
        let stamp = Stamp::new(now_micros(), &value);
        self.write_slot(prim, name, Some(value), Origin::Script, stamp);
        Ok(())
    }

    pub fn remove_slot(&mut self, prim: PrimId, name: &str) {
        let stamp = Stamp::new(now_micros(), &[]);
        self.write_slot(prim, name, None, Origin::Script, stamp);
    }
}

/// Applying entries, from the document or from a compiled prefab. Entries
/// arrive unordered — a child may be seen before its parent — so every path
/// here has to be order-independent.
impl SceneState {
    pub fn apply(&mut self, entry: &Entry) -> Result<(), StateError> {
        let stamp = Stamp::new(entry.timestamp, &entry.value);
        let empty = entry.value.is_empty();

        match key::parse(&entry.key) {
            Some(key::Key::Meta) => {
                if !empty {
                    self.meta = DocMeta::decode(&entry.value)?;
                }
            }
            Some(key::Key::Prop { prim, name }) if name == key::PARENT => {
                let parent = if empty {
                    None
                } else {
                    Some(Parent::decode(&entry.value)?)
                };
                self.write_parent(prim, parent, Origin::Document, Some(stamp));
            }
            Some(key::Key::Prop { prim, name }) if is_slot_name(&name) => {
                let value = if empty {
                    None
                } else {
                    Some(entry.value.clone())
                };
                self.write_slot(prim, &name, value, Origin::Document, stamp);
            }
            Some(key::Key::Prop { prim, name }) => {
                let value = if empty {
                    None
                } else {
                    Some(Property::decode(&entry.value)?)
                };
                self.write_property(prim, &name, value, Origin::Document, stamp);
            }
            None => {}
        }
        Ok(())
    }

    pub fn apply_all<'a>(
        &mut self,
        entries: impl IntoIterator<Item = &'a Entry>,
    ) -> Result<(), StateError> {
        for entry in entries {
            self.apply(entry)?;
        }
        Ok(())
    }

    /// The persistent entry set: everything a save would write. Script-created
    /// prims are transient and absent, which is what keeps a spawn/despawn loop
    /// from accumulating in a namespace that never reclaims.
    #[must_use]
    pub fn entries(&self) -> BTreeMap<String, Vec<u8>> {
        let mut out = BTreeMap::new();
        out.insert(
            key::META.to_owned(),
            self.meta.encode().expect("DocMeta always encodes"),
        );

        for (prim, state) in &self.prims {
            if state.origin != Origin::Document {
                continue;
            }
            let Some(parent) = state.parent else {
                continue;
            };
            out.insert(key::parent(*prim), parent.encode());
            for (name, value) in state.properties() {
                out.insert(key::prop(*prim, name), value.encode());
            }
            for (name, value) in state.slots() {
                out.insert(key::prop(*prim, name), value.to_vec());
            }
        }
        out
    }
}

impl SceneState {
    fn write_parent(
        &mut self,
        prim: PrimId,
        parent: Option<Parent>,
        origin: Origin,
        stamp: Option<Stamp>,
    ) {
        let stamp = stamp.unwrap_or_else(|| {
            Stamp::new(
                now_micros(),
                &parent.map(|p| p.encode()).unwrap_or_default(),
            )
        });

        let state = self
            .prims
            .entry(prim)
            .or_insert_with(|| PrimState::new(origin));
        let old = state.parent;
        if !state.set_parent(parent, stamp) {
            return;
        }
        if origin == Origin::Document {
            state.origin = Origin::Document;
        }

        if let Some(Parent::Prim(old_parent)) = old
            && old != parent
            && let Some(siblings) = self.children.get_mut(&old_parent)
        {
            siblings.remove(&prim);
        }
        if let Some(Parent::Prim(new_parent)) = parent {
            self.children.entry(new_parent).or_default().insert(prim);
        }

        self.refresh(prim);
    }

    fn write_property(
        &mut self,
        prim: PrimId,
        name: &str,
        value: Option<Property>,
        origin: Origin,
        stamp: Stamp,
    ) {
        let state = self
            .prims
            .entry(prim)
            .or_insert_with(|| PrimState::new(origin));
        let changed = match value.clone() {
            Some(value) => state.set_property(name, value, stamp),
            None => state.remove_property(name, stamp),
        };
        if changed && self.realized.contains_key(&prim) {
            self.events.push(SceneEvent::Property {
                prim,
                name: SmolStr::new(name),
                value,
            });
        }
    }

    fn write_slot(
        &mut self,
        prim: PrimId,
        name: &str,
        value: Option<Vec<u8>>,
        origin: Origin,
        stamp: Stamp,
    ) {
        let state = self
            .prims
            .entry(prim)
            .or_insert_with(|| PrimState::new(origin));
        let changed = match value.clone() {
            Some(value) => state.set_slot(name, value, stamp),
            None => state.remove_slot(name, stamp),
        };
        if changed && self.realized.contains_key(&prim) {
            self.events.push(SceneEvent::Slot {
                prim,
                name: SmolStr::new(name),
                value,
            });
        }
    }

    /// Recomputes realization for `root` and, if it changed, everything under
    /// it. Cycles are visited once thanks to `seen`.
    fn refresh(&mut self, root: PrimId) {
        let mut seen = HashSet::new();
        let mut stack = vec![root];

        while let Some(prim) = stack.pop() {
            if !seen.insert(prim) {
                continue;
            }

            let placement = self.placement(prim);
            let previous = self.realized.get(&prim).copied();

            let changed = match placement {
                Placement::Unrealized => {
                    if previous.is_some() {
                        self.realized.remove(&prim);
                        self.events.push(SceneEvent::Unrealized { prim });
                        true
                    } else {
                        false
                    }
                }
                Placement::Root | Placement::Child(_) => {
                    let parent = match placement {
                        Placement::Child(parent) => Some(parent),
                        _ => None,
                    };
                    match previous {
                        None => {
                            self.realized.insert(prim, parent);
                            self.events.push(SceneEvent::Realized { prim, parent });
                            self.emit_contents(prim);
                            true
                        }
                        Some(previous) if previous != parent => {
                            self.realized.insert(prim, parent);
                            self.events.push(SceneEvent::Reparented { prim, parent });
                            true
                        }
                        Some(_) => false,
                    }
                }
            };

            if changed && let Some(children) = self.children.get(&prim) {
                stack.extend(children.iter().copied());
            }
        }
    }

    /// Emits everything a newly realized prim already holds, so a consumer
    /// never has to read state directly to catch up.
    fn emit_contents(&mut self, prim: PrimId) {
        let Some(state) = self.prims.get(&prim) else {
            return;
        };
        let props = state
            .properties()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>();
        let slots = state
            .slots()
            .map(|(name, value)| (name.clone(), value.to_vec()))
            .collect::<Vec<_>>();

        for (name, value) in props {
            self.events.push(SceneEvent::Property {
                prim,
                name,
                value: Some(value),
            });
        }
        for (name, value) in slots {
            self.events.push(SceneEvent::Slot {
                prim,
                name,
                value: Some(value),
            });
        }
    }

    /// LWW parent pointers do not guarantee acyclicity the way Loro's movable
    /// tree did, so a cycle breaks at its greatest-stamped member, which every
    /// peer computes identically from the same entries.
    fn placement(&self, prim: PrimId) -> Placement {
        let Some(state) = self.prims.get(&prim) else {
            return Placement::Unrealized;
        };
        // Already-realized prims stay realized; only new ones are turned away,
        // so a full document keeps converging instead of thrashing.
        if self.realized.len() >= MAX_REALIZED_PRIMS && !self.realized.contains_key(&prim) {
            return Placement::Unrealized;
        }
        let parent = match state.parent {
            None => return Placement::Unrealized,
            Some(Parent::Root) => return Placement::Root,
            Some(Parent::Prim(parent)) => parent,
        };

        let mut chain = vec![prim];
        let mut seen = HashMap::from([(prim, 0usize)]);
        let mut current = parent;
        loop {
            if let Some(&index) = seen.get(&current) {
                let breaker = chain[index..]
                    .iter()
                    .copied()
                    .max_by_key(|id| (self.parent_stamp(*id), *id))
                    .unwrap_or(prim);
                return if breaker == prim {
                    Placement::Root
                } else {
                    Placement::Child(parent)
                };
            }
            if chain.len() >= MAX_PRIM_DEPTH {
                return Placement::Unrealized;
            }
            seen.insert(current, chain.len());
            chain.push(current);

            match self.prims.get(&current).and_then(|s| s.parent) {
                None => return Placement::Unrealized,
                Some(Parent::Root) => return Placement::Child(parent),
                Some(Parent::Prim(next)) => current = next,
            }
        }
    }

    fn parent_stamp(&self, prim: PrimId) -> Stamp {
        self.prims
            .get(&prim)
            .map(PrimState::parent_stamp)
            .unwrap_or_default()
    }
}

fn now_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_micros() as u64)
}
