//! Entity lookups by id, kept current by component hooks so hot paths never
//! scan the world.

use std::hash::Hash;

use bevy::{
    ecs::{
        lifecycle::HookContext,
        world::DeferredWorld,
    },
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::document::HsdDocId;
use hsd::id::DocId;

/// A component that names the id its entity is looked up by.
pub trait Indexed: Component {
    type Key: Copy + Eq + Hash + Send + Sync + 'static;

    fn key(&self) -> Self::Key;
}

/// Every live entity carrying `C`, by its [`Indexed::key`].
#[derive(Resource)]
pub struct Index<C: Indexed>(HashMap<C::Key, Entity>);

impl<C: Indexed> Default for Index<C> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

impl<C: Indexed> Index<C> {
    #[must_use]
    pub fn get(&self, key: C::Key) -> Option<Entity> {
        self.0.get(&key).copied()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// `on_insert` hook. A world without the index resource keeps none.
pub fn insert<C: Indexed>(mut world: DeferredWorld, ctx: HookContext) {
    let Some(key) = world.get::<C>(ctx.entity).map(Indexed::key) else {
        return;
    };
    if let Some(mut index) = world.get_resource_mut::<Index<C>>() {
        index.0.insert(key, ctx.entity);
    }
}

/// `on_discard` hook. Leaves the key alone if another entity has since
/// claimed it.
pub fn discard<C: Indexed>(mut world: DeferredWorld, ctx: HookContext) {
    let Some(key) = world.get::<C>(ctx.entity).map(Indexed::key) else {
        return;
    };
    if let Some(mut index) = world.get_resource_mut::<Index<C>>()
        && index.0.get(&key) == Some(&ctx.entity)
    {
        index.0.remove(&key);
    }
}

/// Every entity carrying an [`HsdDocId`], by document id. Kept by
/// [`index_document`] and [`unindex_document`].
#[derive(Resource, Default)]
pub struct DocIndex(HashMap<DocId, Entity>);

impl DocIndex {
    #[must_use]
    pub fn get(&self, doc: DocId) -> Option<Entity> {
        self.0.get(&doc).copied()
    }
}

pub fn index_document(
    trigger: On<Insert, HsdDocId>,
    ids: Query<&HsdDocId>,
    mut index: ResMut<DocIndex>,
) {
    if let Ok(id) = ids.get(trigger.entity) {
        index.0.insert(id.0, trigger.entity);
    }
}

pub fn unindex_document(
    trigger: On<Discard, HsdDocId>,
    ids: Query<&HsdDocId>,
    mut index: ResMut<DocIndex>,
) {
    if let Ok(id) = ids.get(trigger.entity)
        && index.0.get(&id.0) == Some(&trigger.entity)
    {
        index.0.remove(&id.0);
    }
}

/// `C`'s entity under `key`, if the world keeps an index of `C`.
#[must_use]
pub fn lookup<C: Indexed>(world: &World, key: C::Key) -> Option<Entity> {
    world.get_resource::<Index<C>>()?.get(key)
}
