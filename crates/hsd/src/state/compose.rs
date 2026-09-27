use std::collections::{
    HashMap,
    HashSet,
};

use crate::{
    id::{
        PRIM_ID_BYTES,
        PrimId,
    },
    property::{
        name::PropName,
        value::Value,
    },
    schema::parent::ParentAttr,
    state::{
        HsdState,
        MAX_PRIM_DEPTH,
        MAX_REALIZED_PRIMS,
        entry::{
            Stamp,
            now_millis,
        },
        event::SceneEvent,
        layer::{
            Layer,
            LayerId,
        },
        prim::PrimState,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    Root,
    Child(PrimId),
    Held,
}

/// Scratch space for one [`HsdState::refresh`].
#[derive(Default)]
struct Walk {
    /// The parent chain being walked, in order and as a set.
    order:   Vec<PrimId>,
    members: HashSet<PrimId>,
    /// Depth of each prim this refresh realized under a root.
    depths:  HashMap<PrimId, usize>,
}

impl Walk {
    fn reset(&mut self, prim: PrimId) {
        self.order.clear();
        self.members.clear();
        self.push(prim);
    }

    fn push(&mut self, prim: PrimId) {
        self.order.push(prim);
        self.members.insert(prim);
    }

    /// The chain from `prim` onward, if `prim` is already on it.
    fn cycle_at(&self, prim: PrimId) -> Option<&[PrimId]> {
        if !self.members.contains(&prim) {
            return None;
        }
        let index = self.order.iter().position(|&id| id == prim)?;
        Some(&self.order[index..])
    }
}

/// Now, or one past `held` when the wall clock has not passed it.
fn after(held: Option<Stamp>) -> u64 {
    let now = now_millis();
    held.map_or(now, |held| now.max(held.timestamp.saturating_add(1)))
}

/// The strongest opinion on a key. `Blocked` stops the walk.
fn resolve_property<'a>(layers: &'a [Layer], prim: PrimId, name: &PropName) -> Option<&'a Value> {
    layers
        .iter()
        .rev()
        .find_map(|layer| layer.get(prim)?.property(name))
        .and_then(|opinion| opinion.value())
}

impl HsdState {
    pub(super) const fn layer(&self, id: LayerId) -> &Layer {
        &self.layers[id.idx()]
    }

    pub(super) const fn layer_mut(&mut self, id: LayerId) -> &mut Layer {
        &mut self.layers[id.idx()]
    }

    /// A timestamp for a local write to a property in `layer`, past the stamp
    /// the layer already holds for it even when the wall clock has not moved.
    pub(super) fn local_property_time(&self, layer: LayerId, prim: PrimId, name: &PropName) -> u64 {
        after(
            self.layer(layer)
                .get(prim)
                .and_then(|opinions| opinions.property_stamp(name)),
        )
    }

    pub(super) fn local_parent_stamp(
        &self,
        layer: LayerId,
        prim: PrimId,
        parent: Option<ParentAttr>,
    ) -> Stamp {
        let held = self
            .layer(layer)
            .get(prim)
            .and_then(|opinions| opinions.parent())
            .map(|(_, stamp)| *stamp);
        let timestamp = after(held);
        let Some(parent) = parent else {
            return Stamp::new(timestamp, &[]);
        };
        let mut buf = [0u8; PRIM_ID_BYTES + 1];
        postcard::to_slice(&parent, &mut buf).map_or_else(
            |_| Stamp::new(timestamp, &ParentAttr::to_wire(Some(parent))),
            |encoded| Stamp::of_attribute(timestamp, encoded),
        )
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

    /// Stamps a local write with [`Self::local_parent_stamp`] when `stamp` is
    /// `None`.
    pub(super) fn write_parent(
        &mut self,
        layer: LayerId,
        prim: PrimId,
        parent: Option<ParentAttr>,
        stamp: Option<Stamp>,
    ) {
        let stamp = stamp.unwrap_or_else(|| self.local_parent_stamp(layer, prim, parent));
        if self
            .layer_mut(layer)
            .entry(prim)
            .set_parent(parent.into(), stamp)
        {
            self.settle_parent(prim);
        }
    }

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
            if siblings.is_empty() {
                self.children.remove(&old_parent);
            }
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
        name: &PropName,
        value: Option<Value>,
        stamp: Stamp,
    ) {
        if self
            .layer_mut(layer)
            .entry(prim)
            .set_property(name, value.into(), stamp)
        {
            self.settle_property(prim, name);
        }
    }

    /// Recomposes one property, emitting it if the composed value moved.
    pub(super) fn settle_property(&mut self, prim: PrimId, name: &PropName) {
        let resolved = resolve_property(&self.layers, prim, name);
        let view = self.resolved.entry(prim).or_default();
        if view.property(name) == resolved {
            return;
        }
        let value = resolved.cloned();
        view.set_property(name, value.clone());

        if self.realized.contains_key(&prim) {
            self.events.push(SceneEvent::Property {
                prim,
                name: name.clone(),
                value,
            });
        }
    }

    /// Drops the composed view of a prim no layer holds an opinion on.
    pub(super) fn forget_if_unstated(&mut self, prim: PrimId) {
        if self.layers.iter().all(|layer| layer.get(prim).is_none()) {
            self.resolved.remove(&prim);
        }
    }

    /// Recomputes realization for `root` and its subtree. Descends through
    /// every prim that is or was realized, since a move changes the depth and
    /// cycle membership of everything beneath it.
    fn refresh(&mut self, root: PrimId) {
        let mut seen = HashSet::new();
        let mut stack = vec![root];
        let mut walk = Walk::default();

        while let Some(prim) = stack.pop() {
            if !seen.insert(prim) {
                continue;
            }
            let placement = self.placement(prim, &mut walk);
            let was_realized = self.place(prim, placement);
            if (was_realized || placement != Placement::Held)
                && let Some(children) = self.children.get(&prim)
            {
                stack.extend(children.iter().copied());
            }
        }
    }

    /// Answers whether the prim was realized before.
    fn place(&mut self, prim: PrimId, placement: Placement) -> bool {
        let parent = match placement {
            Placement::Held => {
                let was_realized = self.realized.remove(&prim).is_some();
                if was_realized {
                    self.events.push(SceneEvent::Unrealized { prim });
                }
                return was_realized;
            }
            Placement::Root => None,
            Placement::Child(parent) => Some(parent),
        };
        match self.realized.insert(prim, parent) {
            None => {
                self.events.push(SceneEvent::Realized { prim, parent });
                self.emit_contents(prim);
                false
            }
            Some(previous) => {
                if previous != parent {
                    self.events.push(SceneEvent::Reparented { prim, parent });
                }
                true
            }
        }
    }

    /// Emits every property a newly realized prim already holds.
    pub(super) fn emit_contents(&mut self, prim: PrimId) {
        let Some(state) = self.resolved.get(&prim) else {
            return;
        };
        self.events.extend(
            state
                .properties()
                .map(|(name, value)| SceneEvent::Property {
                    prim,
                    name: name.clone(),
                    value: Some(value.clone()),
                }),
        );
    }

    /// A parent cycle breaks at its greatest-stamped member, which becomes a
    /// root. A prim whose chain reaches a root realizes only under a realized
    /// parent.
    fn placement(&self, prim: PrimId, walk: &mut Walk) -> Placement {
        let Some(state) = self.resolved.get(&prim) else {
            return Placement::Held;
        };
        if self.realized.len() >= MAX_REALIZED_PRIMS && !self.realized.contains_key(&prim) {
            return Placement::Held;
        }
        let parent = match state.parent {
            None => return Placement::Held,
            Some(ParentAttr::Root) => {
                walk.depths.insert(prim, 0);
                return Placement::Root;
            }
            Some(ParentAttr::Prim(parent)) => parent,
        };

        let depth = match walk.depths.get(&parent) {
            Some(&parent_depth) => parent_depth + 1,
            None => match self.walk_to_root(prim, parent, walk) {
                Ok(depth) => depth,
                Err(placement) => return placement,
            },
        };
        if depth >= MAX_PRIM_DEPTH || !self.realized.contains_key(&parent) {
            return Placement::Held;
        }
        walk.depths.insert(prim, depth);
        Placement::Child(parent)
    }

    /// Walks up from `prim` to its root, answering its depth. Ends early with
    /// the placement when the chain is broken, too deep, or cycles.
    fn walk_to_root(
        &self,
        prim: PrimId,
        parent: PrimId,
        walk: &mut Walk,
    ) -> Result<usize, Placement> {
        walk.reset(prim);
        let mut current = parent;
        loop {
            if let Some(cycle) = walk.cycle_at(current) {
                let breaker = cycle
                    .iter()
                    .copied()
                    .max_by_key(|id| (self.parent_stamp(*id), *id));
                return Err(if breaker == Some(prim) {
                    Placement::Root
                } else {
                    Placement::Child(parent)
                });
            }
            if walk.order.len() >= MAX_PRIM_DEPTH {
                return Err(Placement::Held);
            }
            walk.push(current);

            match self.resolved.get(&current).and_then(|s| s.parent) {
                None => return Err(Placement::Held),
                Some(ParentAttr::Root) => return Ok(walk.order.len() - 1),
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
