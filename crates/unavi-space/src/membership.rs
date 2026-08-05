use std::{
    collections::HashMap,
    sync::LazyLock,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdDocId,
};
use hsd::id::DocId;
use iroh_docs::NamespaceId;
use parking_lot::RwLock;

use crate::Space;

/// Maps document -> space it belongs to.
///
/// Keyed by document id, not namespace: a prefab instance belongs to a space
/// and has an id, but has no namespace of its own.
pub static DOC_SPACE_REGISTRY: LazyLock<RwLock<HashMap<DocId, DocId>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[must_use]
pub fn space_doc_id(space: &Space) -> DocId {
    DocId(*space.0.as_bytes())
}

#[derive(Component)]
#[relationship(relationship_target = SpaceMembers)]
pub struct SpaceOwner(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = SpaceOwner)]
pub struct SpaceMembers(Vec<Entity>);

pub fn self_own_space(trigger: On<Add, Space>, spaces: Query<&Space>) {
    let Ok(space) = spaces.get(trigger.entity) else {
        return;
    };
    let id = space_doc_id(space);
    DOC_SPACE_REGISTRY.write().insert(id, id);
}

/// Assigns every unowned document to the space it hangs under.
///
/// A pass rather than an observer: a document is routinely instanced before the
/// document hosting it belongs to a space — a prefab under a document that is
/// only made a space once its entries are read — and an insertion hook fires
/// once, too early, and never again. Retrying each frame also walks nested
/// instances down, one level per pass.
pub fn parent_docs_under_space(
    docs: Query<
        (Entity, &HsdDocId, Option<&ChildOf>),
        (With<Hsd>, Without<Space>, Without<SpaceOwner>),
    >,
    prims: Query<&HsdChild>,
    spaces: Query<(Entity, &HsdDocId), With<Space>>,
    is_space: Query<(), With<Space>>,
    owners: Query<&SpaceOwner>,
    mut commands: Commands,
) {
    for (entity, doc_record, parent) in &docs {
        if let Some(prim) = parent.map(ChildOf::parent)
            && let Ok(doc) = prims.get(prim).map(|c| c.0)
        {
            let space = if is_space.contains(doc) {
                doc
            } else if let Ok(owner) = owners.get(doc) {
                owner.0
            } else {
                continue;
            };
            commands.entity(entity).insert(SpaceOwner(space));
            continue;
        }

        let Some(space_id) = DOC_SPACE_REGISTRY.read().get(&doc_record.0).copied() else {
            continue;
        };
        let Some(space_entity) = spaces
            .iter()
            .find_map(|(e, r)| (r.0 == space_id).then_some(e))
        else {
            continue;
        };
        commands
            .entity(entity)
            .insert((ChildOf(space_entity), SpaceOwner(space_entity)));
    }
}

pub fn register_on_owner_change(
    trigger: On<Insert, SpaceOwner>,
    owners: Query<(&HsdDocId, &SpaceOwner), With<Hsd>>,
    spaces: Query<&Space>,
) {
    let Ok((doc_record, owner)) = owners.get(trigger.entity) else {
        return;
    };
    let Ok(space) = spaces.get(owner.0) else {
        return;
    };
    DOC_SPACE_REGISTRY
        .write()
        .insert(doc_record.0, space_doc_id(space));
}

pub fn deregister_doc_membership(trigger: On<Remove, SpaceOwner>, docs: Query<&HsdDocId>) {
    if let Ok(record) = docs.get(trigger.entity) {
        DOC_SPACE_REGISTRY.write().remove(&record.0);
    }
}

pub fn deregister_space_docs(trigger: On<Remove, Space>, spaces: Query<&Space>) {
    if let Ok(space) = spaces.get(trigger.entity) {
        let id = space_doc_id(space);
        DOC_SPACE_REGISTRY.write().retain(|_, v| *v != id);
    }
}

/// The space a doc belongs to.
#[must_use]
pub fn doc_space(doc: DocId) -> Option<DocId> {
    DOC_SPACE_REGISTRY.read().get(&doc).copied().or_else(|| {
        // Replicas track pinned documents, which are namespace-backed by
        // definition — you pin a namespace into a space.
        crate::state::replicas::space_of(NamespaceId::from(&doc.0)).map(|ns| DocId(*ns.as_bytes()))
    })
}

#[must_use]
pub fn same_space(a: DocId, b: DocId) -> bool {
    match (doc_space(a), doc_space(b)) {
        (Some(x), Some(y)) => x == y,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use bevy_hsd::Prim;
    use hsd::{
        id::PrimId,
        state::SceneState,
    };

    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_observer(self_own_space)
            .add_observer(register_on_owner_change)
            .add_systems(Update, parent_docs_under_space);
        app
    }

    #[test]
    fn instance_adopted_after_host_becomes_a_space() {
        let mut app = app();

        let ns = NamespaceId::from(blake3::hash(b"host-doc").as_bytes());
        let host_id = DocId(*ns.as_bytes());
        let host = app
            .world_mut()
            .spawn((Hsd::new(SceneState::new()), HsdDocId(host_id)))
            .id();

        let prim_id = PrimId::new();
        let prim = app.world_mut().spawn((Prim(prim_id), HsdChild(host))).id();

        let instance_id = DocId::instance(host_id, prim_id);
        let instance = app
            .world_mut()
            .spawn((
                Hsd::new(SceneState::new()),
                HsdDocId(instance_id),
                ChildOf(prim),
            ))
            .id();

        app.update();
        assert!(app.world().get::<SpaceOwner>(instance).is_none());

        app.world_mut().entity_mut(host).insert(Space(ns));
        app.update();

        assert_eq!(
            app.world().get::<SpaceOwner>(instance).map(|o| o.0),
            Some(host)
        );
        assert_eq!(doc_space(instance_id), Some(host_id));
    }
}
