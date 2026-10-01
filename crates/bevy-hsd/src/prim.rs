//! A document's prims, and the cross-prim relationships between them.

use std::collections::BTreeMap;

use bevy::{
    ecs::system::SystemParam,
    platform::collections::{
        HashMap,
        HashSet,
    },
    prelude::*,
};
use hsd::{
    id::PrimId,
    property::name::PropName,
};

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Prim(pub PrimId);

/// The document a prim belongs to.
#[derive(Component)]
#[relationship(relationship_target = Prims)]
pub struct PrimOf(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = PrimOf, linked_spawn)]
pub struct Prims(Vec<Entity>);

/// A document's prims, keyed by id.
#[derive(Component, Default, Debug)]
pub struct PrimIndex {
    entities: HashMap<PrimId, Entity>,
    /// Prims whose relationships name each target. Keyed by id so a target
    /// that does not exist yet is found once it appears.
    sources:  HashMap<PrimId, HashSet<Entity>>,
}

impl PrimIndex {
    #[must_use]
    pub fn get(&self, prim: PrimId) -> Option<Entity> {
        self.entities.get(&prim).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (PrimId, Entity)> + '_ {
        self.entities.iter().map(|(&prim, &entity)| (prim, entity))
    }

    pub(crate) fn insert(&mut self, prim: PrimId, entity: Entity) {
        self.entities.insert(prim, entity);
    }

    pub(crate) fn remove(&mut self, prim: PrimId) -> Option<Entity> {
        self.entities.remove(&prim)
    }

    pub(crate) fn link(&mut self, target: PrimId, source: Entity) {
        self.sources.entry(target).or_default().insert(source);
    }

    pub(crate) fn unlink(&mut self, target: PrimId, source: Entity) {
        let Some(sources) = self.sources.get_mut(&target) else {
            return;
        };
        sources.remove(&source);
        if sources.is_empty() {
            self.sources.remove(&target);
        }
    }

    /// The prim entities whose relationship properties name `target`.
    pub(crate) fn sources(&self, target: PrimId) -> impl Iterator<Item = Entity> + '_ {
        self.sources.get(&target).into_iter().flatten().copied()
    }
}

/// A prim's relationship properties.
#[derive(Component, Default, Debug)]
pub struct HsdRelationships(pub BTreeMap<PropName, PrimId>);

/// Resolves relationships between prim entities of one document.
#[derive(SystemParam)]
pub(crate) struct PrimRelations<'w, 's> {
    docs_of:       Query<'w, 's, &'static PrimOf>,
    indices:       Query<'w, 's, &'static PrimIndex>,
    relationships: Query<'w, 's, &'static HsdRelationships>,
    prims:         Query<'w, 's, &'static Prim>,
}

impl PrimRelations<'_, '_> {
    /// `None` if unset or the target is not in the scene.
    pub(crate) fn target(&self, prim: Entity, name: &PropName) -> Option<Entity> {
        let rels = self.relationships.get(prim).ok()?;
        let target = rels.0.get(name)?;
        let doc = self.docs_of.get(prim).ok()?.0;
        self.indices.get(doc).ok()?.get(*target)
    }

    /// Prims whose relationships name `target`. Empty once `target` despawns.
    pub(crate) fn sources(&self, target: Entity) -> impl Iterator<Item = Entity> + '_ {
        let found = (|| {
            let id = self.prims.get(target).ok()?.0;
            let doc = self.docs_of.get(target).ok()?.0;
            Some(self.indices.get(doc).ok()?.sources(id).collect::<Vec<_>>())
        })();
        found.into_iter().flatten()
    }
}
