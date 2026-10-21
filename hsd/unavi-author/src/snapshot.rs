//! A prim and everything beneath it, as properties, so it can be built again.

use std::collections::{
    HashMap,
    VecDeque,
};

use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    protocol::{
        PrimId,
        key_of,
    },
    wired::{
        core::error::Error,
        scene::{
            document::{
                Document,
                Edit,
                Layer,
            },
            properties::{
                Property,
                PropertyKey,
            },
        },
    },
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimState {
    pub prim:       PrimId,
    pub properties: Vec<Property>,
}

/// Every prim of a subtree, each after its parent, so the first is the root.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot(pub Vec<PrimState>);

impl Snapshot {
    /// `root` and its descendants as `doc` resolves them now, or `None` when
    /// `root` is not in the scene.
    #[must_use]
    pub fn capture(doc: &Document, root: PrimId) -> Option<Self> {
        if !doc.contains(root) {
            return None;
        }
        let mut prims = Vec::new();
        let mut queue = VecDeque::from([root]);
        while let Some(prim) = queue.pop_front() {
            let keys = doc
                .keys(prim)
                .into_iter()
                .map(|key| (prim, key))
                .collect::<Vec<_>>();
            let properties = doc.get_many(&keys).into_iter().flatten().collect();
            prims.push(PrimState { prim, properties });
            queue.extend(doc.children(prim));
        }
        Some(Self(prims))
    }

    /// Every key the snapshot states, for committing it.
    #[must_use]
    pub fn keys(&self) -> Vec<(PrimId, PropertyKey)> {
        self.0
            .iter()
            .flat_map(|state| {
                state
                    .properties
                    .iter()
                    .map(move |property| (state.prim, key_of(property)))
            })
            .collect()
    }

    /// Each prim's `parent` key, which commits its removal once removed.
    #[must_use]
    pub fn parent_keys(&self) -> Vec<(PrimId, PropertyKey)> {
        self.0
            .iter()
            .map(|state| (state.prim, PropertyKey::Parent))
            .collect()
    }

    /// Builds the subtree again in `doc`, in the local layer, under new prim
    /// ids, and rewrites this snapshot to name them, answering each old id
    /// with its new one. A parent or relation naming a prim outside the
    /// subtree keeps that prim.
    pub fn rebuild(&mut self, doc: &Document) -> Result<Vec<(PrimId, PrimId)>, Error> {
        // Every prim exists before any property is set, so a relation to a
        // prim later in the walk finds its new id.
        let mut renamed = HashMap::<PrimId, PrimId>::new();
        for state in &self.0 {
            let parent = state
                .properties
                .iter()
                .find_map(|property| match property {
                    Property::Parent(parent) => *parent,
                    _ => None,
                })
                .map(|parent| renamed.get(&parent).copied().unwrap_or(parent));
            renamed.insert(state.prim, doc.create_prim(Layer::Local, parent)?);
        }
        let rename = |prim: PrimId| renamed.get(&prim).copied().unwrap_or(prim);

        let mut edits = Vec::new();
        for state in &mut self.0 {
            state.prim = rename(state.prim);
            for property in &mut state.properties {
                match property {
                    // `create-prim` already set it.
                    Property::Parent(parent) => {
                        *parent = parent.map(rename);
                        continue;
                    }
                    Property::Relation((_, target)) => *target = rename(*target),
                    _ => {}
                }
                edits.push(Edit::Set((state.prim, property.clone())));
            }
        }
        doc.apply(Layer::Local, &edits)?;
        Ok(renamed.into_iter().collect())
    }

    /// Every prim, each after its parent.
    pub fn prims(&self) -> impl DoubleEndedIterator<Item = PrimId> + '_ {
        self.0.iter().map(|state| state.prim)
    }

    /// Renames `old` to `new` wherever this snapshot names it.
    pub fn rename(&mut self, old: PrimId, new: PrimId) {
        for state in &mut self.0 {
            if state.prim == old {
                state.prim = new;
            }
            for property in &mut state.properties {
                rename_in(property, old, new);
            }
        }
    }
}

/// Renames `old` to `new` where `property` names a prim.
pub fn rename_in(property: &mut Property, old: PrimId, new: PrimId) {
    match property {
        Property::Parent(Some(parent)) if *parent == old => *parent = new,
        Property::Relation((_, target)) if *target == old => *target = new,
        _ => {}
    }
}
