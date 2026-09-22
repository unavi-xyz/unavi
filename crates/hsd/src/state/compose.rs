//! The layer-to-view engine: every write path funnels into `write_*`, which
//! settles the composed view and the events a consumer would apply.

use std::collections::HashSet;

use smol_str::SmolStr;

use crate::{
    attributes::parent::ParentAttr,
    id::{
        PRIM_ID_BYTES,
        PrimId,
    },
    property::Property,
    state::{
        HsdState,
        MAX_PRIM_DEPTH,
        MAX_REALIZED_PRIMS,
        entry::Stamp,
        event::SceneEvent,
        layer::{
            Layer,
            LayerId,
        },
        prim::PrimState,
    },
};

/// Where a prim sits once the tree's integrity rules have been applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    Root,
    Child(PrimId),
    /// Held: the parent chain reaches a prim that does not exist.
    Unrealized,
}

/// The strongest opinion on a key, or `None` where every layer is silent and
/// where the strongest opinion is `Blocked` — `Blocked` stops the walk rather
/// than falling through.
///
/// Takes `layers` rather than `&self` so the caller can hold this borrow
/// against `resolved` at the same time: it is the layers, not the whole state,
/// that this reads.
fn resolve_property<'a>(layers: &'a [Layer], prim: PrimId, name: &str) -> Option<&'a Property> {
    layers
        .iter()
        .rev()
        .find_map(|layer| layer.get(prim)?.property(name))
        .and_then(|opinion| opinion.value())
}

/// The stamp a local parent write gets when the caller has none. Hashing the
/// payload out of a stack buffer avoids `ParentAttr::to_wire`'s two
/// allocations.
fn parent_stamp(parent: Option<ParentAttr>) -> Stamp {
    let Some(parent) = parent else {
        return Stamp::now(&[]);
    };
    let mut buf = [0u8; PRIM_ID_BYTES + 1];
    postcard::to_slice(&parent, &mut buf).map_or_else(
        |_| Stamp::now(&ParentAttr::to_wire(Some(parent))),
        |encoded| Stamp::for_attribute(encoded),
    )
}

impl HsdState {
    pub(super) fn layer(&mut self, id: LayerId) -> &mut Layer {
        &mut self.layers[id.idx()]
    }

    fn resolve_parent(&self, prim: PrimId) -> (Option<ParentAttr>, Stamp) {
        self.layers
            .iter()
            .rev()
            .find_map(|layer| layer.get(prim)?.parent())
            .map_or_else(
                || (None, Stamp::default()),
                |(opinion, stamp)| (opinion.value().copied(), *stamp),
            )
    }

    pub(super) fn write_parent(
        &mut self,
        layer: LayerId,
        prim: PrimId,
        parent: Option<ParentAttr>,
        stamp: Option<Stamp>,
    ) {
        let stamp = stamp.unwrap_or_else(|| parent_stamp(parent));

        if !self
            .layer(layer)
            .entry(prim)
            .set_parent(parent.into(), stamp)
        {
            return;
        }
        self.settle_parent(prim);
    }

    /// Recomposes `prim`'s parent from the stack, reindexes its place among
    /// its siblings, and re-realizes whatever that moved.
    pub(super) fn settle_parent(&mut self, prim: PrimId) {
        let (parent, stamp) = self.resolve_parent(prim);
        let view = self.resolved.entry(prim).or_default();
        let old = view.parent;
        view.set_parent(parent, stamp);

        if let Some(ParentAttr::Prim(old_parent)) = old
            && old != parent
            && let Some(siblings) = self.children.get_mut(&old_parent)
        {
            siblings.remove(&prim);
        }
        if let Some(ParentAttr::Prim(new_parent)) = parent {
            self.children.entry(new_parent).or_default().insert(prim);
        }

        self.refresh(prim);
    }

    pub(super) fn write_property(
        &mut self,
        layer: LayerId,
        prim: PrimId,
        name: &str,
        value: Option<Property>,
        stamp: Stamp,
    ) {
        if !self
            .layer(layer)
            .entry(prim)
            .set_property(name, value.into(), stamp)
        {
            return;
        }
        self.settle_property(prim, name);
    }

    /// Recomposes one property from the stack and emits it if what a consumer
    /// would draw changed.
    ///
    /// A weaker layer writing under a stronger one's opinion costs a resolve
    /// and nothing else: the composed value did not move, so there is nothing
    /// to apply.
    pub(super) fn settle_property(&mut self, prim: PrimId, name: &str) {
        // Compare against the layers by reference, so an unchanged value is
        // never cloned — only the strongest opinion's own bytes are read.
        let resolved = resolve_property(&self.layers, prim, name);
        let view = self.resolved.entry(prim).or_default();
        if view.property(name) == resolved {
            return;
        }
        let value = resolved.cloned();
        view.set_property(name, value.clone());

        if self.realized.contains_key(&prim) {
            let name = view
                .property_key(name)
                .unwrap_or_else(|| SmolStr::new(name));
            self.events.push(SceneEvent::Property { prim, name, value });
        }
    }

    /// Recomputes realization for `root` and, if it changed, everything under
    /// it. Cycles are visited once thanks to `seen`.
    fn refresh(&mut self, root: PrimId) {
        let mut seen = HashSet::new();
        let mut stack = vec![root];
        // Reused across every prim this walk touches, so `placement` does not
        // allocate per prim.
        let mut chain = Vec::new();

        while let Some(prim) = stack.pop() {
            if !seen.insert(prim) {
                continue;
            }

            let placement = self.placement(prim, &mut chain);
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
    pub(super) fn emit_contents(&mut self, prim: PrimId) {
        let Some(state) = self.resolved.get(&prim) else {
            return;
        };
        let values = state
            .properties()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<Vec<_>>();

        for (name, value) in values {
            self.events.push(SceneEvent::Property {
                prim,
                name,
                value: Some(value),
            });
        }
    }

    /// LWW parent pointers can form cycles; the cycle breaks at its
    /// greatest-stamped member, which every peer computes identically.
    fn placement(&self, prim: PrimId, chain: &mut Vec<PrimId>) -> Placement {
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
            Some(ParentAttr::Root) => return Placement::Root,
            Some(ParentAttr::Prim(parent)) => parent,
        };

        // `chain` holds the visited prims in order, so a lookup by index is
        // the whole cycle check — the map it replaces bought nothing on a
        // chain this short and allocated on every call.
        chain.clear();
        chain.push(prim);
        let mut current = parent;
        loop {
            if let Some(index) = chain.iter().position(|&id| id == current) {
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
            chain.push(current);

            match self.resolved.get(&current).and_then(|s| s.parent) {
                None => return Placement::Unrealized,
                Some(ParentAttr::Root) => return Placement::Child(parent),
                Some(ParentAttr::Prim(next)) => current = next,
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
