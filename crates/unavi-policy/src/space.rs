use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdDocId,
};
use hsd::id::DocId;
use iroh_docs::NamespaceId;

use crate::{
    owner::Owner,
    permissions::Permissions,
    registry::Policy,
};

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Space(pub NamespaceId);

impl Space {
    /// The space's own document.
    #[must_use]
    pub fn doc_id(&self) -> DocId {
        DocId(*self.0.as_bytes())
    }
}

#[derive(Component)]
#[relationship(relationship_target = SpaceMembers)]
pub struct SpaceOwner(pub Entity);

/// `linked_spawn` so unloading a space despawns its documents.
#[derive(Component, Default)]
#[relationship_target(relationship = SpaceOwner, linked_spawn)]
pub struct SpaceMembers(Vec<Entity>);

/// Grants a space's own document the space preset and registers it into
/// itself. The only place authority is granted to content the local user did
/// not author.
pub fn register_space(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    policy: Res<Policy>,
    mut commands: Commands,
) {
    let Ok(space) = spaces.get(trigger.entity) else {
        return;
    };
    let id = space.doc_id();
    commands
        .entity(trigger.entity)
        .insert((Permissions::space(), Owner::Space(id)));
    policy.update(id, |record| record.space = Some(id));
}

/// Assigns every unowned document to the space it hangs under.
///
/// A system rather than an observer. A document is often instanced before its
/// host becomes a space.
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

pub fn register_membership(
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

pub fn forget_membership(
    trigger: On<Remove, SpaceOwner>,
    docs: Query<&HsdDocId>,
    policy: Res<Policy>,
) {
    if let Ok(record) = docs.get(trigger.entity) {
        policy.update(record.0, |record| record.space = None);
    }
}

pub fn forget_space(trigger: On<Remove, Space>, spaces: Query<&Space>, policy: Res<Policy>) {
    if let Ok(space) = spaces.get(trigger.entity) {
        policy.forget_space(space.doc_id());
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
    use crate::{
        registry::Record,
        sync,
    };

    /// An app with its own registry.
    fn app() -> (App, Policy) {
        let mut app = App::new();
        app.init_resource::<Policy>()
            .add_observer(register_space)
            .add_observer(register_membership)
            .add_observer(forget_membership)
            .add_observer(forget_space)
            .add_observer(sync::sync_on::<HsdDocId>)
            .add_observer(sync::sync_on::<Permissions>)
            .add_observer(sync::sync_on::<Owner>)
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
    fn entering_a_space_grants_its_own_document_the_space_preset() {
        let (mut app, policy) = app();

        let ns = NamespaceId::from(blake3::hash(b"granted").as_bytes());
        let id = DocId(*ns.as_bytes());
        app.world_mut()
            .spawn((Hsd::new(SceneState::new()), HsdDocId(id), Space(ns)));
        app.update();

        let record = policy.get(id);
        assert_eq!(record.permissions, Permissions::space());
        assert_eq!(record.owner, Some(Owner::Space(id)));
        assert_eq!(record.space, Some(id));
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

    /// A scratch document never gets a `SpaceOwner`.
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
