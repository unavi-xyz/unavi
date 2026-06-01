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

pub static DOC_SPACE_REGISTRY: LazyLock<RwLock<HashMap<Hash, Hash>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Component)]
#[relationship(relationship_target = SpaceMembers)]
pub struct SpaceOwner(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = SpaceOwner)]
pub struct SpaceMembers(Vec<Entity>);

pub fn self_own_space(trigger: On<Add, Space>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(SpaceOwner(trigger.entity));
}

pub fn register_doc_membership(
    changed: Query<(&HsdRecordId, &SpaceOwner), (With<Hsd>, Changed<SpaceOwner>)>,
    spaces: Query<&HsdRecordId, With<Space>>,
) {
    if changed.is_empty() {
        return;
    }
    let mut reg = DOC_SPACE_REGISTRY.write();
    for (doc_record, owner) in &changed {
        let Ok(space_record) = spaces.get(owner.0) else {
            continue;
        };
        reg.insert(doc_record.0, space_record.0);
    }
}

pub fn deregister_doc_membership(trigger: On<Remove, SpaceOwner>, docs: Query<&HsdRecordId>) {
    let Ok(record) = docs.get(trigger.entity) else {
        return;
    };
    DOC_SPACE_REGISTRY.write().remove(&record.0);
}

#[must_use]
pub fn doc_space(doc: Hash) -> Option<Hash> {
    DOC_SPACE_REGISTRY.read().get(&doc).copied()
}

#[must_use]
pub fn same_space(a: Hash, b: Hash) -> bool {
    let reg = DOC_SPACE_REGISTRY.read();
    match (reg.get(&a), reg.get(&b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}
