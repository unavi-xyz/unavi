//! The live scene document, as the runtime holds it.
//!
//! `HsdState` composes opinions through a layer stack into the resolved view
//! every consumer draws, realizes prims, and emits [`SceneEvent`]s. Split by
//! concern: `compose` is the layer-to-view engine, `apply` ingests entries,
//! `commit` promotes live opinions, `overrides` holds what referencing
//! documents say, and `write` is the local authoring surface.

use std::collections::{
    BTreeMap,
    BTreeSet,
    HashMap,
};

use thiserror::Error;

use crate::{
    attributes::Attribute,
    id::PrimId,
    meta::DocMeta,
    property::PropertyError,
    state::{
        event::SceneEvent,
        layer::{
            Layer,
            LayerId,
            Overrides,
        },
        prim::PrimState,
    },
};

mod apply;
mod commit;
mod compose;
mod overrides;
mod write;

pub mod entry;
pub mod event;
pub mod layer;
pub mod opinion;
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
/// per-document quota is charged. Prims past the cap stay held, exactly as an
/// orphan does, and realize if room frees up.
pub const MAX_REALIZED_PRIMS: usize = 100_000;

/// The layers a [`HsdState::commit`] may promote opinions into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitTarget {
    /// The document layer: what a save writes to the namespace, so a
    /// promotion here is durable.
    Document,
    /// The overrides the document referencing this one holds over it, named by
    /// the reference site. Where an edit to content authored elsewhere lands,
    /// and durable in the referencing document rather than this one — so a
    /// commit here answers the entries that document has to hold.
    Override { site: PrimId },
    /// The session layer: this session only, replicated by nothing yet.
    /// The fallback for a commit from a client holding no durable key.
    Session,
}

#[derive(Debug)]
pub struct HsdState {
    meta:              DocMeta,
    /// Weakest first, so iterating forwards composes and iterating backwards
    /// finds the strongest opinion on a key.
    layers:            BTreeMap<LayerId, Layer>,
    /// What this document says about the prims of the documents its own prims
    /// reference, keyed by reference site. Durable here and installed into the
    /// referenced document, which is the only place it composes.
    overrides:         HashMap<PrimId, Overrides>,
    /// Bumped on every write to `overrides`, so a realizer can tell whether
    /// what it installed into a child is still current.
    overrides_version: u64,
    /// The composed view every reader sees, recomputed per written key. A
    /// cache: only [`Self::resolve_parent`] and its siblings write it.
    resolved:          HashMap<PrimId, PrimState>,
    /// Parent id to children, indexed over *resolved* parents and including
    /// parents that do not exist yet, which is what lets an orphan be picked
    /// up when its parent arrives.
    children:          HashMap<PrimId, BTreeSet<PrimId>>,
    /// Realized prims and their effective parent, `None` for a document root.
    realized:          HashMap<PrimId, Option<PrimId>>,
    events:            Vec<SceneEvent>,
    /// Write boundaries currently open. A script tick can be suspended between
    /// any two host calls, so its events are withheld until it closes.
    ticks:             usize,
    /// Where the oldest open boundary started writing.
    tick_start:        usize,
}

impl Default for HsdState {
    fn default() -> Self {
        Self::new()
    }
}

impl HsdState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            meta:              DocMeta::default(),
            layers:            LayerId::ALL
                .into_iter()
                .map(|id| (id, Layer::default()))
                .collect(),
            overrides:         HashMap::new(),
            overrides_version: 0,
            resolved:          HashMap::new(),
            children:          HashMap::new(),
            realized:          HashMap::new(),
            events:            Vec::new(),
            ticks:             0,
            tick_start:        0,
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
