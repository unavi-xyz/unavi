use std::{
    collections::HashMap,
    sync::LazyLock,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdRecordId,
};
use blake3::Hash;
use parking_lot::RwLock;

use crate::Space;

/// Maps document -> space it belongs to.
pub static DOC_SPACE_REGISTRY: LazyLock<RwLock<HashMap<Hash, Hash>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Component)]
#[relationship(relationship_target = SpaceMembers)]
pub struct SpaceOwner(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = SpaceOwner)]
pub struct SpaceMembers(Vec<Entity>);

/// Declares the space a freshly spawned, unparented doc should belong to.
#[derive(Component)]
pub struct DocSpaceHint(pub Hash);

pub fn self_own_space(trigger: On<Add, Space>, spaces: Query<&Space>) {
    let Ok(space) = spaces.get(trigger.entity) else {
        return;
    };
    DOC_SPACE_REGISTRY.write().insert(space.0, space.0);
}

pub fn parent_doc_under_space(
    trigger: On<Insert, HsdRecordId>,
    docs: Query<&DocSpaceHint, (With<Hsd>, Without<Space>, Without<ChildOf>)>,
    spaces: Query<(Entity, &HsdRecordId), With<Space>>,
    mut commands: Commands,
) {
    let Ok(hint) = docs.get(trigger.entity) else {
        return;
    };
    let Some(space_entity) = spaces
        .iter()
        .find_map(|(e, r)| (r.0 == hint.0).then_some(e))
    else {
        return;
    };
    commands
        .entity(trigger.entity)
        .insert((ChildOf(space_entity), SpaceOwner(space_entity)))
        .remove::<DocSpaceHint>();
}

pub fn register_on_owner_change(
    trigger: On<Insert, SpaceOwner>,
    owners: Query<(&HsdRecordId, &SpaceOwner), With<Hsd>>,
    spaces: Query<&Space>,
) {
    let Ok((doc_record, owner)) = owners.get(trigger.entity) else {
        return;
    };
    let Ok(space) = spaces.get(owner.0) else {
        return;
    };
    DOC_SPACE_REGISTRY.write().insert(doc_record.0, space.0);
}

pub fn deregister_doc_membership(trigger: On<Remove, SpaceOwner>, docs: Query<&HsdRecordId>) {
    if let Ok(record) = docs.get(trigger.entity) {
        DOC_SPACE_REGISTRY.write().remove(&record.0);
    }
}

pub fn deregister_space_docs(trigger: On<Remove, Space>, spaces: Query<&Space>) {
    if let Ok(space) = spaces.get(trigger.entity) {
        DOC_SPACE_REGISTRY.write().retain(|_, v| *v != space.0);
    }
}

/// The space a doc belongs to.
#[must_use]
pub fn doc_space(doc: Hash) -> Option<Hash> {
    DOC_SPACE_REGISTRY
        .read()
        .get(&doc)
        .copied()
        .or_else(|| crate::state::peer::space_of(doc))
}

#[must_use]
pub fn same_space(a: Hash, b: Hash) -> bool {
    match (doc_space(a), doc_space(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}
