//! The layer-to-view engine: every write path funnels into `write_*`, which
//! settles the composed view and the events a consumer would apply.

use std::collections::{
    HashMap,
    HashSet,
};

use smol_str::SmolStr;

use crate::{
    id::PrimId,
    property::{
        Parent,
        Property,
    },
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

impl HsdState {
    pub(super) fn layer(&mut self, id: LayerId) -> &mut Layer {
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

    pub(super) fn write_parent(
        &mut self,
        layer: LayerId,
        prim: PrimId,
        parent: Option<Parent>,
        stamp: Option<Stamp>,
    ) {
        let stamp =
            stamp.unwrap_or_else(|| Stamp::now(&parent.map(|p| p.encode()).unwrap_or_default()));

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
        let resolved = self.resolve_property(prim, name);
        let view = self.resolved.entry(prim).or_default();
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

    pub(super) fn write_slot(
        &mut self,
        layer: LayerId,
        prim: PrimId,
        name: &str,
        value: Option<Vec<u8>>,
        stamp: Stamp,
    ) {
        if !self
            .layer(layer)
            .entry(prim)
            .set_slot(name, value.into(), stamp)
        {
            return;
        }
        self.settle_slot(prim, name);
    }

    pub(super) fn settle_slot(&mut self, prim: PrimId, name: &str) {
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
    pub(super) fn emit_contents(&mut self, prim: PrimId) {
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
