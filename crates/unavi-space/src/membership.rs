//! Which space each document belongs to, and which document composed it.
//!
//! Observers keep the [`Policy`] ledger's records in step with the scene
//! graph: a document hanging under a space is a member of it, and one
//! instanced at a reference site is hosted by the document that composed it.

use bevy::prelude::*;
use bevy_hsd::{
    HsdSystems,
    document::{
        Hsd,
        HsdDocId,
        Unplaced,
    },
    prim::PrimOf,
};
use hsd::id::DocId;
use iroh_docs::NamespaceId;
use unavi_policy::Policy;

/// Keeps space membership and document hosts recorded in [`Policy`].
pub struct MembershipPlugin;

impl Plugin for MembershipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Policy>()
            .add_observer(register_document)
            .add_observer(forget_document)
            .add_observer(register_space)
            .add_observer(register_membership)
            .add_observer(forget_membership)
            .add_observer(forget_space)
            .add_systems(Update, parent_docs_under_space.before(HsdSystems));
    }
}

/// A loaded space, rooted at its own document.
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

/// The space a document belongs to.
#[derive(Component)]
#[relationship(relationship_target = SpaceMembers)]
pub struct SpaceOwner(pub Entity);

/// `linked_spawn` so unloading a space despawns its documents.
#[derive(Component, Default)]
#[relationship_target(relationship = SpaceOwner, linked_spawn)]
pub struct SpaceMembers(Vec<Entity>);

/// Registers a space's own document into itself, so everything hanging under
/// it resolves the same space.
pub fn register_space(trigger: On<Add, Space>, spaces: Query<&Space>, policy: Res<Policy>) {
    if let Ok(space) = spaces.get(trigger.entity) {
        let id = space.doc_id();
        policy.update(id, |record| record.space = Some(id));
    }
}

/// Assigns every unowned document to the space it hangs under.
///
/// A system rather than an observer. A document is often instanced before its
/// host becomes a space.
pub fn parent_docs_under_space(
    docs: Query<
        (Entity, &HsdDocId, Option<&ChildOf>),
        (
            With<Hsd>,
            Without<Unplaced>,
            Without<Space>,
            Without<SpaceOwner>,
        ),
    >,
    prims: Query<&PrimOf>,
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

/// Records the document that composed `doc`, once it has an id to key on.
pub fn register_document(
    trigger: On<Insert, HsdDocId>,
    docs: Query<(&HsdDocId, Option<&ChildOf>)>,
    prims: Query<&PrimOf>,
    ids: Query<&HsdDocId>,
    policy: Res<Policy>,
) {
    let Ok((doc, parent)) = docs.get(trigger.entity) else {
        return;
    };
    let Some(host) = host_of(parent, &prims, &ids) else {
        // A document composed some other way — one a script created — states
        // its own host before it is spawned, and the prim chain has nothing to
        // say about it. Clearing it here would orphan it from its author.
        return;
    };
    policy.update(doc.0, |record| record.host = Some(host));
}

/// The document that composed this one in, for a reference site.
fn host_of(
    parent: Option<&ChildOf>,
    prims: &Query<&PrimOf>,
    ids: &Query<&HsdDocId>,
) -> Option<DocId> {
    let prim = parent.map(ChildOf::parent)?;
    let host = prims.get(prim).ok()?.0;
    ids.get(host).ok().map(|id| id.0)
}

/// Drops a despawned document's record. Keyed off the document id, which
/// every document has.
pub fn forget_document(trigger: On<Remove, HsdDocId>, docs: Query<&HsdDocId>, policy: Res<Policy>) {
    if let Ok(doc) = docs.get(trigger.entity) {
        policy.forget_document(doc.0);
    }
}

#[cfg(test)]
mod tests {
    use bevy_hsd::prim::Prim;
    use hsd::{
        id::PrimId,
        state::HsdState,
    };
    use unavi_policy::ledger::Record;

    use super::*;

    /// An app with its own ledger.
    fn app() -> (App, Policy) {
        let mut app = App::new();
        app.add_plugins(MembershipPlugin);
        let policy = app.world().resource::<Policy>().clone();
        (app, policy)
    }

    #[test]
    fn scene_reference_adopted_after_host_becomes_a_space() {
        let (mut app, policy) = app();

        let ns = NamespaceId::from(blake3::hash(b"host-doc").as_bytes());
        let host_id = DocId(*ns.as_bytes());
        let host = app
            .world_mut()
            .spawn((Hsd::new(HsdState::new()), HsdDocId(host_id)))
            .id();

        let prim_id = PrimId::new();
        let prim = app.world_mut().spawn((Prim(prim_id), PrimOf(host))).id();

        let site_id = DocId::site(host_id, prim_id);
        let child = app
            .world_mut()
            .spawn((Hsd::new(HsdState::new()), HsdDocId(site_id), ChildOf(prim)))
            .id();

        app.update();
        assert!(app.world().get::<SpaceOwner>(child).is_none());

        app.world_mut().entity_mut(host).insert(Space(ns));
        app.update();

        assert_eq!(
            app.world().get::<SpaceOwner>(child).map(|o| o.0),
            Some(host)
        );
        assert_eq!(policy.get(site_id).space, Some(host_id));

        app.world_mut().entity_mut(host).despawn();
    }

    #[test]
    fn entering_a_space_registers_its_own_document_into_itself() {
        let (mut app, policy) = app();

        let ns = NamespaceId::from(blake3::hash(b"granted").as_bytes());
        let id = DocId(*ns.as_bytes());
        app.world_mut()
            .spawn((Hsd::new(HsdState::new()), HsdDocId(id), Space(ns)));
        app.update();

        assert_eq!(policy.get(id).space, Some(id));
    }

    #[test]
    fn unloading_a_space_takes_its_documents() {
        let (mut app, _policy) = app();

        let ns = NamespaceId::from(blake3::hash(b"space-doc").as_bytes());
        let space = app
            .world_mut()
            .spawn((Hsd::new(HsdState::new()), HsdDocId(DocId(*ns.as_bytes()))))
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
            .spawn((Hsd::new(HsdState::new()), HsdDocId(id)))
            .id();
        policy.update(id, |record| record.space = Some(DocId([32; 32])));

        app.world_mut().entity_mut(doc).despawn();

        assert_eq!(policy.get(id), Record::default());
    }

    #[test]
    fn a_scene_reference_records_the_host_that_composed_it() {
        let (mut app, policy) = app();
        let host_id = DocId([23; 32]);

        let host = app
            .world_mut()
            .spawn((Hsd::new(HsdState::new()), HsdDocId(host_id)))
            .id();
        let prim_id = PrimId::new();
        let prim = app.world_mut().spawn((Prim(prim_id), PrimOf(host))).id();

        let site_id = DocId::site(host_id, prim_id);
        app.world_mut()
            .spawn((Hsd::new(HsdState::new()), HsdDocId(site_id), ChildOf(prim)));

        assert_eq!(policy.get(site_id).host, Some(host_id));
        assert_eq!(
            policy.root(site_id),
            host_id,
            "an instance resolves its author through the document that \
             composed it"
        );
    }

    #[test]
    fn a_host_stated_before_the_spawn_survives_registration() {
        let (mut app, policy) = app();
        let (parent, child) = (DocId([24; 32]), DocId([25; 32]));

        policy.update(child, |record| record.host = Some(parent));
        app.world_mut()
            .spawn((Hsd::new(HsdState::new()), HsdDocId(child)));

        assert_eq!(
            policy.get(child).host,
            Some(parent),
            "a script-created document hangs off no prim, so the prim chain \
             must not answer for it"
        );
    }

    #[test]
    fn a_despawned_document_leaves_no_record() {
        let (mut app, policy) = app();
        let id = DocId([21; 32]);

        let entity = app
            .world_mut()
            .spawn((Hsd::new(HsdState::new()), HsdDocId(id)))
            .id();
        policy.update(id, |record| record.space = Some(id));

        app.world_mut().entity_mut(entity).despawn();
        assert_eq!(policy.get(id), Record::default());
    }
}
