use bevy::prelude::*;
use bevy_hsd::{
    HsdChild,
    HsdDocId,
};
use hsd::id::DocId;

use crate::{
    document::DocumentPolicy,
    reach::Reach,
    registry::Policy,
};

/// Everything the world can say about one document, read off its entity.
///
/// Every field arrives on its own schedule, so each trigger re-reads all of
/// them; a registration that ran on only one event left components added
/// before the document had an id unregistered.
type DocumentQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static HsdDocId,
        Option<&'static DocumentPolicy>,
        Option<&'static Reach>,
        Option<&'static ChildOf>,
    ),
>;

fn sync(
    policy: &Policy,
    entity: Entity,
    docs: &DocumentQuery,
    prims: &Query<&HsdChild>,
    ids: &Query<&HsdDocId>,
) {
    let Ok((doc, document, reach, parent)) = docs.get(entity) else {
        return;
    };
    let host = host_of(parent, prims, ids);
    let inherited = host.map(|host| policy.get(host).policy);

    policy.update(doc.0, |record| {
        // A prefab instance runs with the policy of the document that composed
        // it in; anything else with a policy of its own states it directly.
        if let Some(document) = document.copied().or(inherited) {
            record.policy = document;
        }
        if let Some(reach) = reach {
            record.reach = *reach;
        }
        record.host = host;
    });
}

/// The document that composed this one in, for a prefab instance: up to the
/// prim carrying the slot, then to the document that prim belongs to.
fn host_of(
    parent: Option<&ChildOf>,
    prims: &Query<&HsdChild>,
    ids: &Query<&HsdDocId>,
) -> Option<DocId> {
    let prim = parent.map(ChildOf::parent)?;
    let host = prims.get(prim).ok()?.0;
    ids.get(host).ok().map(|id| id.0)
}

pub fn sync_on_doc_id(
    trigger: On<Insert, HsdDocId>,
    docs: DocumentQuery,
    prims: Query<&HsdChild>,
    ids: Query<&HsdDocId>,
    policy: Res<Policy>,
) {
    sync(&policy, trigger.entity, &docs, &prims, &ids);
}

pub fn sync_on_policy(
    trigger: On<Insert, DocumentPolicy>,
    docs: DocumentQuery,
    prims: Query<&HsdChild>,
    ids: Query<&HsdDocId>,
    policy: Res<Policy>,
) {
    sync(&policy, trigger.entity, &docs, &prims, &ids);
}

pub fn sync_on_reach(
    trigger: On<Insert, Reach>,
    docs: DocumentQuery,
    prims: Query<&HsdChild>,
    ids: Query<&HsdDocId>,
    policy: Res<Policy>,
) {
    sync(&policy, trigger.entity, &docs, &prims, &ids);
}

/// Keyed off the document id itself: every document has one, so none can leave
/// its record behind by never having acquired some other marker.
pub fn forget_document(trigger: On<Remove, HsdDocId>, docs: Query<&HsdDocId>, policy: Res<Policy>) {
    if let Ok(doc) = docs.get(trigger.entity) {
        policy.forget(doc.0);
    }
}

#[cfg(test)]
mod tests {
    use bevy_hsd::{
        Hsd,
        Prim,
    };
    use hsd::{
        id::PrimId,
        state::SceneState,
    };

    use super::*;
    use crate::{
        tier::Tier,
        trust::Trust,
    };

    /// An app with its own registry, so nothing here shares state with its
    /// neighbours.
    fn app() -> (App, Policy) {
        let mut app = App::new();
        app.init_resource::<Policy>()
            .add_observer(sync_on_doc_id)
            .add_observer(sync_on_policy)
            .add_observer(sync_on_reach)
            .add_observer(forget_document);
        let policy = app.world().resource::<Policy>().clone();
        (app, policy)
    }

    /// The shell is spawned with its policy and its reach alongside `LoadHsd`,
    /// and only learns its document id once a namespace has been minted.
    #[test]
    fn a_document_stated_before_it_had_an_id_still_registers() {
        let (mut app, policy) = app();
        let id = DocId([21; 32]);

        let entity = app
            .world_mut()
            .spawn((DocumentPolicy::system(), Reach::own_only()))
            .id();
        assert_eq!(
            policy.get(id).reach,
            Reach::default(),
            "nothing keys the record until the document has an id"
        );

        app.world_mut()
            .entity_mut(entity)
            .insert((Hsd::new(SceneState::new()), HsdDocId(id)));

        assert_eq!(policy.get(id).reach, Reach::own_only());
        assert_eq!(policy.get(id).policy.tier, Tier::System);

        app.world_mut().entity_mut(entity).despawn();
        assert_eq!(
            policy.get(id).policy.tier,
            Tier::Untrusted,
            "a despawned document must not leave its grant behind"
        );
    }

    #[test]
    fn raising_the_rung_after_registration_takes_effect() {
        let (mut app, policy) = app();
        let id = DocId([22; 32]);

        let entity = app
            .world_mut()
            .spawn((Hsd::new(SceneState::new()), HsdDocId(id)))
            .id();
        assert_eq!(policy.get(id).reach, Reach::default());

        app.world_mut().entity_mut(entity).insert(Reach {
            writes_from: Trust::Trusted,
        });
        assert_eq!(
            policy.get(id).reach.writes_from,
            Trust::Trusted,
            "a document that changes its mind must not be ignored"
        );
    }

    #[test]
    fn an_instance_records_its_host_and_inherits_its_policy() {
        let (mut app, policy) = app();
        let host_id = DocId([23; 32]);

        let host = app
            .world_mut()
            .spawn((
                Hsd::new(SceneState::new()),
                HsdDocId(host_id),
                DocumentPolicy::space(),
            ))
            .id();
        let prim_id = PrimId::new();
        let prim = app.world_mut().spawn((Prim(prim_id), HsdChild(host))).id();

        let instance_id = DocId::instance(host_id, prim_id);
        app.world_mut().spawn((
            Hsd::new(SceneState::new()),
            HsdDocId(instance_id),
            ChildOf(prim),
        ));

        assert_eq!(policy.get(instance_id).host, Some(host_id));
        assert_eq!(policy.get(instance_id).policy.tier, Tier::Space);
    }
}
