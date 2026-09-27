//! A document composed through its layer stack into the view its prims
//! realize from, with the [`SceneEvent`]s describing each change.

use std::collections::{
    BTreeSet,
    HashMap,
};

use thiserror::Error;

use crate::{
    format::meta::DocMeta,
    id::PrimId,
    property::{
        Payload,
        Property,
        name::PropName,
        value::PropertyError,
    },
    state::{
        event::SceneEvent,
        layer::{
            Layer,
            LayerId,
        },
        prim::PrimState,
    },
};

mod apply;
mod commit;
mod compose;
mod reference;
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
    #[error("{0} is written through set_parent")]
    Reserved(PropName),
    #[error("property {0}")]
    Value(#[from] PropertyError),
    #[error("postcard {0}")]
    Postcard(#[from] postcard::Error),
}

/// Deepest parent chain a prim may realize under.
/// Deeper prims are still held in state.
pub const MAX_PRIM_DEPTH: usize = 512;

/// Most prims one document may realize at once.
/// Prims past the cap are held until room frees up.
pub const MAX_REALIZED_PRIMS: usize = 100_000;

/// The layer a [`HsdState::commit`] promotes opinions into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitTarget {
    /// Written back by a save.
    Document,
    /// Durable in the document referencing this one through `site`.
    Override { site: PrimId },
    /// Held for this session only.
    Session,
}

#[derive(Debug)]
pub struct HsdState {
    meta:               DocMeta,
    /// Indexed by [`LayerId`], weakest first.
    layers:             [Layer; LayerId::ALL.len()],
    /// This document's opinions about the prims of the documents it
    /// references, keyed by reference site. They compose only once installed
    /// into the referenced document.
    references:         HashMap<PrimId, Layer>,
    references_version: u64,
    resolved:           HashMap<PrimId, PrimState>,
    /// Keyed by resolved parent, including parents that do not exist yet.
    children:           HashMap<PrimId, BTreeSet<PrimId>>,
    /// Realized prims and their parent, `None` for a root.
    realized:           HashMap<PrimId, Option<PrimId>>,
    events:             Vec<SceneEvent>,
    ticks:              usize,
    /// Where the oldest open write boundary started writing.
    tick_start:         usize,
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
            meta:               DocMeta::default(),
            layers:             LayerId::ALL.map(|_| Layer::default()),
            references:         HashMap::new(),
            references_version: 0,
            resolved:           HashMap::new(),
            children:           HashMap::new(),
            realized:           HashMap::new(),
            events:             Vec::new(),
            ticks:              0,
            tick_start:         0,
        }
    }

    #[must_use]
    pub fn get(&self, prim: PrimId) -> Option<&PrimState> {
        self.resolved.get(&prim)
    }

    /// Whether some layer states a live parent for the prim. An existing prim
    /// may still be held rather than realized.
    #[must_use]
    pub fn exists(&self, prim: PrimId) -> bool {
        self.resolved.get(&prim).is_some_and(|s| s.parent.is_some())
    }

    #[must_use]
    pub fn is_realized(&self, prim: PrimId) -> bool {
        self.realized.contains_key(&prim)
    }

    /// `None` for a root or a prim that is not realized.
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

    #[must_use]
    pub fn attribute<A: Property>(&self, prim: PrimId) -> Option<Result<A, postcard::Error>> {
        self.payload(prim, &A::NAME)
    }

    /// The field `name` decoded as `V`, `None` when it holds no attribute.
    #[must_use]
    pub fn payload<V: Payload>(
        &self,
        prim: PrimId,
        name: &PropName,
    ) -> Option<Result<V, postcard::Error>> {
        let payload = self.get(prim)?.property(name)?.as_attribute()?;
        Some(V::decode(payload))
    }

    #[must_use]
    pub fn relationship(&self, prim: PrimId, name: &PropName) -> Option<PrimId> {
        self.get(prim)?.property(name)?.as_relationship()
    }

    /// Opens a write boundary. Events written until the matching
    /// [`Self::close_tick`] are withheld from [`Self::drain_events`].
    /// Boundaries nest.
    pub const fn open_tick(&mut self) {
        if self.ticks == 0 {
            self.tick_start = self.events.len();
        }
        self.ticks += 1;
    }

    /// An unmatched close is ignored.
    pub const fn close_tick(&mut self) {
        self.ticks = self.ticks.saturating_sub(1);
    }

    /// Events written outside any open boundary.
    pub fn drain_events(&mut self) -> Vec<SceneEvent> {
        if self.ticks == 0 {
            return std::mem::take(&mut self.events);
        }
        let complete = self.events.drain(..self.tick_start).collect();
        self.tick_start = 0;
        complete
    }

    /// Replaces pending events with a description of the realized scene,
    /// parents before children. The description drains even while a boundary
    /// is open.
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

        self.tick_start = self.events.len();
    }
}
