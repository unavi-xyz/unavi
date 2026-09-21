use std::collections::{
    BTreeMap,
    BTreeSet,
    HashMap,
    HashSet,
};

use smol_str::SmolStr;
use thiserror::Error;
use web_time::{
    SystemTime,
    UNIX_EPOCH,
};

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
        layer::{
            Layer,
            LayerId,
        },
        opinion::Origin,
        prim::PrimState,
    },
};

pub mod entry;
pub mod event;
mod layer;
pub mod opinion;
pub mod prim;
pub mod save;

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
/// per-document quota is charged. Prims past the cap stay held, exactly as an
/// orphan does, and realize if room frees up.
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
    meta:       DocMeta,
    /// Weakest first, so iterating forwards composes and iterating backwards
    /// finds the strongest opinion on a key.
    layers:     BTreeMap<LayerId, Layer>,
    /// The composed view every reader sees, recomputed per written key. A
    /// cache: only [`Self::resolve_parent`] and its siblings write it.
    resolved:   HashMap<PrimId, PrimState>,
    /// Parent id to children, indexed over *resolved* parents and including
    /// parents that do not exist yet, which is what lets an orphan be picked
    /// up when its parent arrives.
    children:   HashMap<PrimId, BTreeSet<PrimId>>,
    /// Realized prims and their effective parent, `None` for a document root.
    realized:   HashMap<PrimId, Option<PrimId>>,
    events:     Vec<SceneEvent>,
    /// Write boundaries currently open. A script tick can be suspended between
    /// any two host calls, so its events are withheld until it closes.
    ticks:      usize,
    /// Where the oldest open boundary started writing.
    tick_start: usize,
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
            meta:       DocMeta::default(),
            layers:     LayerId::ALL
                .into_iter()
                .map(|id| (id, Layer::default()))
                .collect(),
            resolved:   HashMap::new(),
            children:   HashMap::new(),
            realized:   HashMap::new(),
            events:     Vec::new(),
            ticks:      0,
            tick_start: 0,
        }
    }

    #[must_use]
    pub const fn meta(&self) -> DocMeta {
        self.meta
    }

    #[must_use]
    pub fn get(&self, prim: PrimId) -> Option<&PrimState> {
        self.resolved.get(&prim)
    }

    /// Whether a prim exists: its `parent/` property resolves to a live entry.
    /// Existence is not realization — an existing prim may still be held.
    #[must_use]
    pub fn exists(&self, prim: PrimId) -> bool {
        self.resolved.get(&prim).is_some_and(|s| s.parent.is_some())
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

    /// Opens a write boundary. Everything written until the matching
    /// [`Self::close_tick`] is withheld from [`Self::drain_events`].
    ///
    /// Boundaries nest by count rather than replacing one another, so two
    /// writers on one document hold their events until both are done.
    pub const fn open_tick(&mut self) {
        if self.ticks == 0 {
            self.tick_start = self.events.len();
        }
        self.ticks += 1;
    }

    /// Saturates rather than underflowing: an unmatched close is a bug in the
    /// caller, not a reason to release a boundary someone else is holding.
    pub const fn close_tick(&mut self) {
        self.ticks = self.ticks.saturating_sub(1);
    }

    #[must_use]
    pub const fn is_ticking(&self) -> bool {
        self.ticks > 0
    }

    /// Events belonging to finished writes.
    ///
    /// A tick still in flight keeps its tail: a prim whose creating tick has
    /// not set its transform yet would otherwise be drawn at the origin until
    /// the tick resumes.
    pub fn drain_events(&mut self) -> Vec<SceneEvent> {
        if self.ticks == 0 {
            return std::mem::take(&mut self.events);
        }
        let complete = self.events.drain(..self.tick_start).collect();
        self.tick_start = 0;
        complete
    }

    /// Replaces pending events with a full description of the realized scene,
    /// so a consumer attaching to an already-built state gets everything.
    /// Parents are emitted before their children.
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

        // A consumer attaching mid-tick needs the scene as it stands now, so
        // the description itself drains; only writes after it are withheld.
        self.tick_start = self.events.len();
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
            LayerId::Document,
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
        self.write_parent(
            LayerId::Document,
            prim,
            Some(parent),
            Origin::Document,
            None,
        );
    }

    pub fn set_parent(&mut self, prim: PrimId, parent: Parent) -> Result<(), StateError> {
        if !self.exists(prim) {
            return Err(StateError::UnknownPrim(prim));
        }
        self.write_parent(LayerId::Document, prim, Some(parent), Origin::Script, None);
        Ok(())
    }

    pub fn remove_prim(&mut self, prim: PrimId) {
        if !self.layer(LayerId::Document).contains(prim) {
            return;
        }
        self.write_parent(LayerId::Document, prim, None, Origin::Script, None);
        self.layer(LayerId::Document).remove(prim);
        self.resolved.remove(&prim);
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
        self.write_property(
            LayerId::Document,
            prim,
            name,
            Some(value),
            Origin::Script,
            stamp,
        );
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
        self.write_property(LayerId::Document, prim, name, None, Origin::Script, stamp);
    }

    pub fn set_slot(&mut self, prim: PrimId, name: &str, value: Vec<u8>) -> Result<(), StateError> {
        if !key::is_valid_name(name) {
            return Err(StateError::Name(name.to_owned()));
        }
        let stamp = Stamp::new(now_micros(), &value);
        self.write_slot(
            LayerId::Document,
            prim,
            name,
            Some(value),
            Origin::Script,
            stamp,
        );
        Ok(())
    }

    pub fn remove_slot(&mut self, prim: PrimId, name: &str) {
        let stamp = Stamp::new(now_micros(), &[]);
        self.write_slot(LayerId::Document, prim, name, None, Origin::Script, stamp);
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
                self.write_parent(
                    LayerId::Document,
                    prim,
                    parent,
                    Origin::Document,
                    Some(stamp),
                );
            }
            Some(key::Key::Prop { prim, name }) if is_slot_name(&name) => {
                let value = if empty {
                    None
                } else {
                    Some(entry.value.clone())
                };
                self.write_slot(
                    LayerId::Document,
                    prim,
                    &name,
                    value,
                    Origin::Document,
                    stamp,
                );
            }
            Some(key::Key::Prop { prim, name }) => {
                let value = if empty {
                    None
                } else {
                    Some(Property::decode(&entry.value)?)
                };
                self.write_property(
                    LayerId::Document,
                    prim,
                    &name,
                    value,
                    Origin::Document,
                    stamp,
                );
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

        let Some(document) = self.layers.get(&LayerId::Document) else {
            return out;
        };
        for (prim, opinions) in document.prims() {
            if opinions.origin != Origin::Document {
                continue;
            }
            let Some(parent) = opinions.parent().and_then(|(o, _)| o.value()) else {
                continue;
            };
            out.insert(key::parent(prim), parent.encode());
            for (name, value) in opinions.set_properties() {
                out.insert(key::prop(prim, name), value.encode());
            }
            for (name, value) in opinions.set_slots() {
                out.insert(key::prop(prim, name), value.to_vec());
            }
        }
        out
    }
}

impl SceneState {
    fn layer(&mut self, id: LayerId) -> &mut Layer {
        self.layers.entry(id).or_default()
    }

    /// The strongest opinion on a key, or `None` where every layer is silent
    /// and where the strongest opinion is `Blocked` — `Blocked` stops the walk
    /// rather than falling through.
    fn resolve_property(&self, prim: PrimId, name: &str) -> Option<Property> {
        self.layers
            .values()
            .rev()
            .find_map(|layer| layer.get(prim)?.property(name))
            .and_then(|opinion| opinion.value().cloned())
    }

    fn resolve_slot(&self, prim: PrimId, name: &str) -> Option<Vec<u8>> {
        self.layers
            .values()
            .rev()
            .find_map(|layer| layer.get(prim)?.slot(name))
            .and_then(|opinion| opinion.value().cloned())
    }

    fn resolve_parent(&self, prim: PrimId) -> (Option<Parent>, Stamp) {
        self.layers
            .values()
            .rev()
            .find_map(|layer| layer.get(prim)?.parent())
            .map_or_else(
                || (None, Stamp::default()),
                |(opinion, stamp)| (opinion.value().copied(), *stamp),
            )
    }

    fn write_parent(
        &mut self,
        layer: LayerId,
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

        let opinions = self.layer(layer).entry(prim, origin);
        if !opinions.set_parent(parent.into(), stamp) {
            return;
        }
        if origin == Origin::Document {
            opinions.origin = Origin::Document;
        }
        self.settle_parent(prim);
    }

    /// Recomposes `prim`'s parent from the stack, reindexes its place among
    /// its siblings, and re-realizes whatever that moved.
    fn settle_parent(&mut self, prim: PrimId) {
        let (parent, stamp) = self.resolve_parent(prim);
        let view = self.resolved.entry(prim).or_default();
        let old = view.parent;
        view.set_parent(parent, stamp);

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
        layer: LayerId,
        prim: PrimId,
        name: &str,
        value: Option<Property>,
        origin: Origin,
        stamp: Stamp,
    ) {
        if !self
            .layer(layer)
            .entry(prim, origin)
            .set_property(name, value.into(), stamp)
        {
            return;
        }

        let resolved = self.resolve_property(prim, name);
        let view = self.resolved.entry(prim).or_default();
        // A weaker layer writing under a stronger one's opinion costs a
        // resolve and nothing else: the composed value did not move, so there
        // is nothing for a consumer to apply.
        if view.property(name) == resolved.as_ref() {
            return;
        }
        view.set_property(name, resolved.clone());

        if self.realized.contains_key(&prim) {
            self.events.push(SceneEvent::Property {
                prim,
                name: SmolStr::new(name),
                value: resolved,
            });
        }
    }

    fn write_slot(
        &mut self,
        layer: LayerId,
        prim: PrimId,
        name: &str,
        value: Option<Vec<u8>>,
        origin: Origin,
        stamp: Stamp,
    ) {
        if !self
            .layer(layer)
            .entry(prim, origin)
            .set_slot(name, value.into(), stamp)
        {
            return;
        }

        let resolved = self.resolve_slot(prim, name);
        let view = self.resolved.entry(prim).or_default();
        if view.slot(name) == resolved.as_deref() {
            return;
        }
        view.set_slot(name, resolved.clone());

        if self.realized.contains_key(&prim) {
            self.events.push(SceneEvent::Slot {
                prim,
                name: SmolStr::new(name),
                value: resolved,
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
        let Some(state) = self.resolved.get(&prim) else {
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

    /// LWW parent pointers can form cycles; the cycle breaks at its
    /// greatest-stamped member, which every peer computes identically.
    fn placement(&self, prim: PrimId) -> Placement {
        let Some(state) = self.resolved.get(&prim) else {
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

            match self.resolved.get(&current).and_then(|s| s.parent) {
                None => return Placement::Unrealized,
                Some(Parent::Root) => return Placement::Child(parent),
                Some(Parent::Prim(next)) => current = next,
            }
        }
    }

    fn parent_stamp(&self, prim: PrimId) -> Stamp {
        self.resolved
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{
        material::{
            BINDING,
            MaterialAttr,
        },
        name::NameAttr,
        xform::XformAttr,
    };

    fn prim(n: u8) -> PrimId {
        PrimId([n; 16])
    }

    fn root_entry(id: PrimId, timestamp: u64) -> Entry {
        Entry::bytes(key::parent(id), Parent::Root.encode(), timestamp)
    }

    fn child_entry(id: PrimId, parent: PrimId, timestamp: u64) -> Entry {
        Entry::bytes(key::parent(id), Parent::Prim(parent).encode(), timestamp)
    }

    fn attr_entry<A: Attribute>(id: PrimId, value: &A, timestamp: u64) -> Entry {
        Entry::bytes(
            key::prop(id, A::KEY),
            Property::Attribute(value.encode().expect("encode")).encode(),
            timestamp,
        )
    }

    fn tombstone(key: String, timestamp: u64) -> Entry {
        Entry::bytes(key, Vec::new(), timestamp)
    }

    fn apply(state: &mut SceneState, entries: &[Entry]) {
        state.apply_all(entries).expect("apply");
    }

    fn shape(state: &SceneState) -> BTreeMap<PrimId, Option<PrimId>> {
        state
            .prims()
            .map(|prim| (prim, state.parent(prim)))
            .collect()
    }

    #[test]
    fn realizes_a_root_and_its_child() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[root_entry(prim(1), 1), child_entry(prim(2), prim(1), 2)],
        );

        assert_eq!(state.roots(), vec![prim(1)]);
        assert_eq!(state.children(prim(1)), vec![prim(2)]);
        assert_eq!(state.parent(prim(2)), Some(prim(1)));
    }

    #[test]
    fn entry_order_does_not_change_the_result() {
        let entries = [
            attr_entry(prim(3), &NameAttr("leaf".into()), 4),
            child_entry(prim(3), prim(2), 3),
            child_entry(prim(2), prim(1), 2),
            root_entry(prim(1), 1),
        ];

        let mut forward = SceneState::new();
        apply(&mut forward, &entries);

        let mut reversed = SceneState::new();
        let mut flipped = entries.clone();
        flipped.reverse();
        apply(&mut reversed, &flipped);

        assert_eq!(shape(&forward), shape(&reversed));
        assert_eq!(forward.entries(), reversed.entries());
        assert_eq!(
            reversed
                .attribute::<NameAttr>(prim(3))
                .expect("name")
                .expect("decode"),
            NameAttr("leaf".into())
        );
    }

    #[test]
    fn an_orphan_is_held_not_reparented_to_the_root() {
        let mut state = SceneState::new();
        apply(&mut state, &[child_entry(prim(2), prim(1), 2)]);

        assert!(state.exists(prim(2)));
        assert!(!state.is_realized(prim(2)));
        assert_eq!(state.roots().len(), 0);
    }

    #[test]
    fn an_orphan_realizes_with_its_properties_when_its_parent_arrives() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                child_entry(prim(2), prim(1), 2),
                attr_entry(prim(2), &XformAttr::default(), 3),
            ],
        );
        assert_eq!(state.drain_events().len(), 0);

        apply(&mut state, &[root_entry(prim(1), 1)]);
        let events = state.drain_events();

        assert!(events.contains(&SceneEvent::Realized {
            prim:   prim(2),
            parent: Some(prim(1)),
        }));
        assert!(events.iter().any(|e| matches!(
            e,
            SceneEvent::Property { prim: p, name, .. } if *p == prim(2) && name == XformAttr::KEY
        )));
    }

    #[test]
    fn a_property_on_an_unrealized_prim_emits_nothing() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[attr_entry(prim(9), &NameAttr("held".into()), 1)],
        );
        assert_eq!(state.drain_events().len(), 0);
    }

    #[test]
    fn a_cycle_breaks_at_its_greatest_stamp_regardless_of_order() {
        let entries = [
            child_entry(prim(1), prim(3), 10),
            child_entry(prim(2), prim(1), 20),
            child_entry(prim(3), prim(2), 30),
        ];

        let mut forward = SceneState::new();
        apply(&mut forward, &entries);

        let mut shuffled = SceneState::new();
        apply(
            &mut shuffled,
            &[entries[2].clone(), entries[0].clone(), entries[1].clone()],
        );

        assert_eq!(shape(&forward), shape(&shuffled));
        assert_eq!(forward.roots(), vec![prim(3)]);
        assert_eq!(forward.parent(prim(1)), Some(prim(3)));
        assert_eq!(forward.parent(prim(2)), Some(prim(1)));
    }

    #[test]
    fn a_prim_hanging_off_a_cycle_is_realized_under_its_own_parent() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                child_entry(prim(1), prim(2), 10),
                child_entry(prim(2), prim(1), 20),
                child_entry(prim(3), prim(1), 30),
            ],
        );

        assert_eq!(state.roots(), vec![prim(2)]);
        assert_eq!(state.parent(prim(3)), Some(prim(1)));
    }

    #[test]
    fn a_cross_author_tombstone_removes_a_prim_written_by_someone_else() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                child_entry(prim(2), prim(1), 2),
                attr_entry(prim(2), &NameAttr("gone".into()), 3),
            ],
        );
        state.drain_events();

        apply(&mut state, &[tombstone(key::parent(prim(2)), 10)]);

        assert!(!state.exists(prim(2)));
        assert!(!state.is_realized(prim(2)));
        assert_eq!(state.children(prim(1)), Vec::new());
        assert_eq!(
            state.drain_events(),
            vec![SceneEvent::Unrealized { prim: prim(2) }]
        );
    }

    #[test]
    fn deleting_a_prim_holds_its_descendants_rather_than_dropping_them() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                child_entry(prim(2), prim(1), 2),
                child_entry(prim(3), prim(2), 3),
            ],
        );

        apply(&mut state, &[tombstone(key::parent(prim(2)), 10)]);
        assert!(!state.is_realized(prim(3)));
        assert!(state.exists(prim(3)));

        apply(&mut state, &[child_entry(prim(2), prim(1), 20)]);
        assert!(state.is_realized(prim(3)));
    }

    #[test]
    fn an_older_entry_never_overwrites_a_newer_one() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                attr_entry(prim(1), &NameAttr("new".into()), 100),
                attr_entry(prim(1), &NameAttr("old".into()), 50),
            ],
        );

        assert_eq!(
            state
                .attribute::<NameAttr>(prim(1))
                .expect("name")
                .expect("decode"),
            NameAttr("new".into())
        );
    }

    #[test]
    fn concurrent_writes_at_one_timestamp_resolve_the_same_way_both_orders() {
        let a = attr_entry(prim(1), &NameAttr("alpha".into()), 7);
        let b = attr_entry(prim(1), &NameAttr("beta".into()), 7);

        let mut forward = SceneState::new();
        apply(
            &mut forward,
            &[root_entry(prim(1), 1), a.clone(), b.clone()],
        );

        let mut reversed = SceneState::new();
        apply(&mut reversed, &[root_entry(prim(1), 1), b, a]);

        assert_eq!(
            forward
                .attribute::<NameAttr>(prim(1))
                .expect("name")
                .expect("decode"),
            reversed
                .attribute::<NameAttr>(prim(1))
                .expect("name")
                .expect("decode"),
        );
    }

    #[test]
    fn an_unknown_attribute_round_trips_untouched() {
        let payload = vec![0xCA, 0xFE, 0xBA, 0xBE];
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                Entry::bytes(
                    key::prop(prim(1), "shader_graph"),
                    Property::Attribute(payload.clone()).encode(),
                    2,
                ),
            ],
        );

        let entries = state.entries();
        let stored = entries
            .get(&key::prop(prim(1), "shader_graph"))
            .expect("entry");
        assert_eq!(stored, &Property::Attribute(payload).encode());
    }

    #[test]
    fn relationships_and_attributes_share_one_namespace() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[root_entry(prim(1), 1), root_entry(prim(2), 1)],
        );

        state
            .set_attribute(prim(1), &MaterialAttr::default())
            .expect("attribute");
        state
            .set_relationship(prim(1), BINDING, prim(2))
            .expect("relationship");

        let prim_state = state.get(prim(1)).expect("prim");
        assert!(
            prim_state
                .property("material")
                .expect("attr")
                .as_attribute()
                .is_some()
        );
        assert_eq!(
            prim_state.property(BINDING).expect("rel").as_relationship(),
            Some(prim(2))
        );
    }

    #[test]
    fn script_created_prims_are_absent_from_the_save_set() {
        let mut state = SceneState::new();
        apply(&mut state, &[root_entry(prim(1), 1)]);

        let scratch = state.create_prim(Some(prim(1)));
        state
            .set_attribute(scratch, &NameAttr("transient".into()))
            .expect("attribute");

        assert!(state.is_realized(scratch));
        let entries = state.entries();
        assert!(entries.contains_key(&key::parent(prim(1))));
        assert!(!entries.contains_key(&key::parent(scratch)));
    }

    #[test]
    fn a_document_prim_edited_by_a_script_stays_persistent() {
        let mut state = SceneState::new();
        apply(&mut state, &[root_entry(prim(1), 1)]);
        state
            .set_attribute(prim(1), &NameAttr("edited".into()))
            .expect("attribute");

        let entries = state.entries();
        assert!(
            entries.contains_key(&key::prop(prim(1), NameAttr::KEY)),
            "a script editing a document prim must not make it transient"
        );
    }

    #[test]
    fn a_slot_is_tracked_as_inline_bytes() {
        let mut state = SceneState::new();
        let payload = vec![9; 1024];
        apply(&mut state, &[root_entry(prim(1), 1)]);
        state.drain_events();

        apply(
            &mut state,
            &[Entry::new(
                key::prop(prim(1), "mesh:POSITION"),
                payload.clone(),
                2,
            )],
        );

        assert_eq!(
            state.get(prim(1)).expect("prim").slot("mesh:POSITION"),
            Some(payload.as_slice())
        );
        assert!(state.drain_events().contains(&SceneEvent::Slot {
            prim:  prim(1),
            name:  "mesh:POSITION".into(),
            value: Some(payload),
        }));
    }

    #[test]
    fn a_zero_size_slot_entry_reads_as_absence() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                Entry::new(key::prop(prim(1), "script"), vec![1; 64], 2),
                Entry::new(key::prop(prim(1), "script"), Vec::new(), 3),
            ],
        );

        assert_eq!(state.get(prim(1)).expect("prim").slot("script"), None);
    }

    #[test]
    fn reparenting_emits_one_event_and_moves_the_subtree() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                root_entry(prim(2), 1),
                child_entry(prim(3), prim(1), 2),
                child_entry(prim(4), prim(3), 3),
            ],
        );
        state.drain_events();

        apply(&mut state, &[child_entry(prim(3), prim(2), 10)]);

        assert_eq!(state.children(prim(1)), Vec::new());
        assert_eq!(state.children(prim(2)), vec![prim(3)]);
        assert_eq!(state.parent(prim(4)), Some(prim(3)));
        assert_eq!(
            state.drain_events(),
            vec![SceneEvent::Reparented {
                prim:   prim(3),
                parent: Some(prim(2)),
            }]
        );
    }

    #[test]
    fn removing_a_property_emits_an_absent_value() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                attr_entry(prim(1), &XformAttr::default(), 2),
            ],
        );
        state.drain_events();

        apply(
            &mut state,
            &[tombstone(key::prop(prim(1), XformAttr::KEY), 3)],
        );

        assert_eq!(
            state.drain_events(),
            vec![SceneEvent::Property {
                prim:  prim(1),
                name:  XformAttr::KEY.into(),
                value: None,
            }]
        );
    }

    #[test]
    fn the_save_set_round_trips_through_a_fresh_state() {
        let mut original = SceneState::new();
        apply(
            &mut original,
            &[
                root_entry(prim(1), 1),
                child_entry(prim(2), prim(1), 2),
                attr_entry(prim(2), &NameAttr("kept".into()), 3),
                Entry::new(key::prop(prim(2), "script"), vec![7; 32], 4),
            ],
        );

        let mut restored = SceneState::new();
        let entries = original
            .entries()
            .into_iter()
            .map(|(key, value)| Entry {
                key,
                value,
                timestamp: 100,
            })
            .collect::<Vec<_>>();
        apply(&mut restored, &entries);

        assert_eq!(shape(&original), shape(&restored));
        assert_eq!(original.entries(), restored.entries());
    }

    #[test]
    fn nesting_past_the_depth_cap_is_not_realized() {
        let mut state = SceneState::new();

        let deep = MAX_PRIM_DEPTH + 8;
        let ids = (0..deep)
            .map(|i| {
                let mut bytes = [0u8; 32];
                bytes[..8].copy_from_slice(&(i as u64).to_be_bytes());
                PrimId::from_digest(&bytes)
            })
            .collect::<Vec<_>>();

        state.apply(&root_entry(ids[0], 0)).expect("root");
        for (i, window) in ids.windows(2).enumerate() {
            state
                .apply(&child_entry(window[1], window[0], i as u64 + 1))
                .expect("child");
        }

        assert!(state.is_realized(ids[0]), "the root realizes");
        assert!(
            state.is_realized(ids[MAX_PRIM_DEPTH - 1]),
            "prims within the cap realize"
        );
        assert!(
            !state.is_realized(ids[deep - 1]),
            "prims past the cap are held"
        );
    }

    #[test]
    fn an_open_tick_withholds_its_own_writes() {
        let mut state = SceneState::new();
        state.open_tick();
        apply(&mut state, &[root_entry(prim(1), 1)]);

        assert!(
            state.drain_events().is_empty(),
            "a prim whose creating tick has not positioned it yet must not be \
             drawn at the origin"
        );

        state.close_tick();
        assert!(
            state.drain_events().contains(&SceneEvent::Realized {
                prim:   prim(1),
                parent: None,
            }),
            "closing the tick releases it"
        );
    }

    #[test]
    fn writes_made_before_a_tick_opened_still_drain() {
        let mut state = SceneState::new();
        apply(&mut state, &[root_entry(prim(1), 1)]);
        state.open_tick();
        apply(&mut state, &[root_entry(prim(2), 2)]);

        let events = state.drain_events();
        assert!(events.contains(&SceneEvent::Realized {
            prim:   prim(1),
            parent: None,
        }));
        assert!(
            !events.contains(&SceneEvent::Realized {
                prim:   prim(2),
                parent: None,
            }),
            "only the open tick's tail is held back"
        );
        state.close_tick();
    }

    #[test]
    fn a_prim_and_its_properties_leave_together() {
        let mut state = SceneState::new();
        state.open_tick();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                attr_entry(prim(1), &XformAttr::default(), 2),
            ],
        );
        assert_eq!(state.drain_events().len(), 0);
        state.close_tick();

        let events = state.drain_events();
        assert!(events.contains(&SceneEvent::Realized {
            prim:   prim(1),
            parent: None,
        }));
        assert!(
            events.iter().any(|e| matches!(
                e,
                SceneEvent::Property { prim: p, name, .. }
                    if *p == prim(1) && name == XformAttr::KEY
            )),
            "the transform arrives in the same drain as the prim it belongs to"
        );
    }

    #[test]
    fn boundaries_nest_so_two_writers_both_have_to_finish() {
        let mut state = SceneState::new();
        state.open_tick();
        state.open_tick();
        apply(&mut state, &[root_entry(prim(1), 1)]);

        state.close_tick();
        assert!(
            state.drain_events().is_empty(),
            "one writer finishing does not release another's partial work"
        );
        assert!(state.is_ticking());

        state.close_tick();
        assert!(!state.is_ticking());
        assert_ne!(state.drain_events().len(), 0);
    }

    #[test]
    fn an_unmatched_close_does_not_underflow() {
        let mut state = SceneState::new();
        state.close_tick();
        assert!(!state.is_ticking());
        state.open_tick();
        apply(&mut state, &[root_entry(prim(1), 1)]);
        assert!(state.drain_events().is_empty(), "the boundary still holds");
        state.close_tick();
    }

    #[test]
    fn a_consumer_attaching_mid_tick_gets_the_scene_as_it_stands() {
        let mut state = SceneState::new();
        apply(&mut state, &[root_entry(prim(1), 1)]);
        state.drain_events();

        state.open_tick();
        state.resync();
        assert!(
            state.drain_events().contains(&SceneEvent::Realized {
                prim:   prim(1),
                parent: None,
            }),
            "a resync is a description of now, not part of anyone's tick"
        );

        apply(&mut state, &[root_entry(prim(2), 2)]);
        assert!(
            state.drain_events().is_empty(),
            "writes after the resync are still the open tick's"
        );
        state.close_tick();
    }

    /// Scripts are not routed into the runtime layer yet, so these reach it
    /// the way that routing will.
    fn runtime_property(state: &mut SceneState, prim: PrimId, name: &str, value: Option<Property>) {
        let stamp = Stamp::new(
            now_micros(),
            &value.as_ref().map(Property::encode).unwrap_or_default(),
        );
        state.write_property(LayerId::Runtime, prim, name, value, Origin::Script, stamp);
    }

    fn name_attr(value: &str) -> Property {
        Property::Attribute(NameAttr(value.into()).encode().expect("encode"))
    }

    fn name_of(state: &SceneState, prim: PrimId) -> Option<String> {
        Some(state.attribute::<NameAttr>(prim)?.expect("decodes").0)
    }

    #[test]
    fn a_runtime_opinion_shadows_the_document_beneath_it() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                attr_entry(prim(1), &NameAttr("document".into()), 2),
            ],
        );
        state.drain_events();

        runtime_property(
            &mut state,
            prim(1),
            NameAttr::KEY,
            Some(name_attr("runtime")),
        );

        assert_eq!(name_of(&state, prim(1)).as_deref(), Some("runtime"));
        assert_eq!(
            state.drain_events(),
            vec![SceneEvent::Property {
                prim:  prim(1),
                name:  SmolStr::new(NameAttr::KEY),
                value: Some(name_attr("runtime")),
            }]
        );
    }

    #[test]
    fn a_document_write_under_a_runtime_opinion_emits_nothing() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                attr_entry(prim(1), &NameAttr("document".into()), 2),
            ],
        );
        runtime_property(
            &mut state,
            prim(1),
            NameAttr::KEY,
            Some(name_attr("runtime")),
        );
        state.drain_events();

        apply(
            &mut state,
            &[attr_entry(prim(1), &NameAttr("later".into()), 3)],
        );

        assert_eq!(name_of(&state, prim(1)).as_deref(), Some("runtime"));
        assert_eq!(
            state.drain_events(),
            Vec::new(),
            "the composed value did not move, so there is nothing to apply; a \
             regression here floods the ECS every frame"
        );
    }

    #[test]
    fn a_blocked_opinion_hides_the_document_value_without_dropping_it() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                attr_entry(prim(1), &NameAttr("document".into()), 2),
            ],
        );
        let saved = state.entries();
        state.drain_events();

        runtime_property(&mut state, prim(1), NameAttr::KEY, None);

        assert_eq!(
            name_of(&state, prim(1)),
            None,
            "Blocked resolves to absent and stops; the weaker value must not \
             show through"
        );
        assert_eq!(
            state.drain_events(),
            vec![SceneEvent::Property {
                prim:  prim(1),
                name:  SmolStr::new(NameAttr::KEY),
                value: None,
            }]
        );
        assert_eq!(
            state.entries(),
            saved,
            "blocking is an opinion of a live layer, not an edit to the document"
        );
    }

    #[test]
    fn a_runtime_write_leaves_the_save_set_byte_identical() {
        let mut state = SceneState::new();
        apply(
            &mut state,
            &[
                root_entry(prim(1), 1),
                attr_entry(prim(1), &NameAttr("document".into()), 2),
            ],
        );
        let saved = state.entries();

        runtime_property(
            &mut state,
            prim(1),
            NameAttr::KEY,
            Some(name_attr("runtime")),
        );
        runtime_property(&mut state, prim(1), "scratch", Some(name_attr("new key")));

        assert_eq!(
            state.entries(),
            saved,
            "this is the guarantee sync_document could not make, and the whole \
             reason it had to freeze a document"
        );
    }

    #[test]
    fn a_prim_the_runtime_layer_alone_states_exists_and_reparents() {
        let mut state = SceneState::new();
        apply(&mut state, &[root_entry(prim(1), 1)]);
        state.drain_events();

        state.write_parent(
            LayerId::Runtime,
            prim(9),
            Some(Parent::Prim(prim(1))),
            Origin::Script,
            None,
        );

        assert!(state.exists(prim(9)));
        assert!(state.is_realized(prim(9)));
        assert_eq!(state.children(prim(1)), vec![prim(9)]);

        state.write_parent(
            LayerId::Runtime,
            prim(9),
            Some(Parent::Root),
            Origin::Script,
            None,
        );

        assert_eq!(state.parent(prim(9)), None);
        assert_eq!(state.children(prim(1)), Vec::new());
        assert!(
            !state.entries().contains_key(&key::parent(prim(9))),
            "a prim only a live layer states never reaches the save set"
        );
    }
}
