use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdDocId,
};

use crate::{
    registry::Policy,
    space::Space,
};

#[derive(Component)]
#[relationship(relationship_target = SpaceMembers)]
pub struct SpaceOwner(pub Entity);

/// `linked_spawn` so unloading a space takes its documents with it. A member is
/// only sometimes a descendant of the space, and one that outlives it keeps
/// simulating with nothing left to stand on.
#[derive(Component, Default)]
#[relationship_target(relationship = SpaceOwner, linked_spawn)]
pub struct SpaceMembers(Vec<Entity>);

pub fn self_own_space(trigger: On<Add, Space>, spaces: Query<&Space>, policy: Res<Policy>) {
    let Ok(space) = spaces.get(trigger.entity) else {
        return;
    };
    let id = space.doc_id();
    policy.update(id, |record| record.space = Some(id));
}

/// Assigns every unowned document to the space it hangs under.
///
/// A pass rather than an observer: a document is routinely instanced before its
/// host becomes a space, and an insertion hook fires too early.
pub fn parent_docs_under_space(
    docs: Query<
        (Entity, &HsdDocId, Option<&ChildOf>),
        (With<Hsd>, Without<Space>, Without<SpaceOwner>),
    >,
    prims: Query<&HsdChild>,
    spaces: Query<(Entity, &HsdDocId), With<Space>>,
    is_space: Query<(), With<Space>>,
    owners: Query<&SpaceOwner>,
    policy: Res<Policy>,
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

        let Some(space_id) = policy.get(doc_record.0).space else {
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
    policy: Res<Policy>,
) {
    let Ok((doc_record, owner)) = owners.get(trigger.entity) else {
        return;
    };
    let Ok(space) = spaces.get(owner.0) else {
        return;
    };
    policy.update(doc_record.0, |record| record.space = Some(space.doc_id()));
}

pub fn deregister_doc_membership(
    trigger: On<Remove, SpaceOwner>,
    docs: Query<&HsdDocId>,
    policy: Res<Policy>,
) {
    if let Ok(record) = docs.get(trigger.entity) {
        policy.update(record.0, |record| record.space = None);
    }
}

pub fn deregister_space_docs(
    trigger: On<Remove, Space>,
    spaces: Query<&Space>,
    policy: Res<Policy>,
) {
    if let Ok(space) = spaces.get(trigger.entity) {
        policy.forget_space(space.doc_id());
    }
}

#[cfg(test)]
mod tests {
    use bevy_hsd::Prim;
    use hsd::{
        id::{
            DocId,
            PrimId,
        },
        state::SceneState,
    };
    use iroh_docs::NamespaceId;

    use super::*;
    use crate::{
        registry::Record,
        sync,
    };

    /// An app with its own registry, so nothing here shares state with its
    /// neighbours.
    fn app() -> (App, Policy) {
        let mut app = App::new();
        app.init_resource::<Policy>()
            .add_observer(self_own_space)
            .add_observer(register_on_owner_change)
            .add_observer(deregister_doc_membership)
            .add_observer(deregister_space_docs)
            .add_observer(sync::sync_on_doc_id)
            .add_observer(sync::forget_document)
            .add_systems(Update, parent_docs_under_space);
        let policy = app.world().resource::<Policy>().clone();
        (app, policy)
    }

    #[test]
    fn instance_adopted_after_host_becomes_a_space() {
        let (mut app, policy) = app();

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
        assert_eq!(policy.get(instance_id).space, Some(host_id));

        app.world_mut().entity_mut(host).despawn();
    }

    #[test]
    fn unloading_a_space_takes_its_documents() {
        let (mut app, _policy) = app();

        let ns = NamespaceId::from(blake3::hash(b"space-doc").as_bytes());
        let space = app
            .world_mut()
            .spawn((Hsd::new(SceneState::new()), HsdDocId(DocId(*ns.as_bytes()))))
            .id();
        app.world_mut().entity_mut(space).insert(Space(ns));

        let doc = app.world_mut().spawn(SpaceOwner(space)).id();

        app.world_mut().entity_mut(space).despawn();
        app.update();

        assert!(app.world().get_entity(doc).is_err());
    }

    /// Scratch documents are spawned with an id and never given a
    /// `SpaceOwner`; the record must still drop on despawn.
    #[test]
    fn a_document_that_never_joined_a_space_still_drops_its_record() {
        let (mut app, policy) = app();
        let id = DocId([31; 32]);

        let doc = app
            .world_mut()
            .spawn((Hsd::new(SceneState::new()), HsdDocId(id)))
            .id();
        policy.update(id, |record| record.space = Some(DocId([32; 32])));

        app.world_mut().entity_mut(doc).despawn();

        assert_eq!(policy.get(id), Record::default());
    }
}
