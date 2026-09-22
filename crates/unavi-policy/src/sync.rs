use bevy::prelude::*;
use bevy_hsd::{
    HsdChild,
    HsdDocId,
};
use hsd::id::DocId;

use crate::registry::Policy;

/// Records the document that composed `doc`, once it has an id to key on.
pub fn register_document(
    trigger: On<Insert, HsdDocId>,
    docs: Query<(&HsdDocId, Option<&ChildOf>)>,
    prims: Query<&HsdChild>,
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
    prims: &Query<&HsdChild>,
    ids: &Query<&HsdDocId>,
) -> Option<DocId> {
    let prim = parent.map(ChildOf::parent)?;
    let host = prims.get(prim).ok()?.0;
    ids.get(host).ok().map(|id| id.0)
}

/// Keyed off the document id, which every document has.
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
        state::HsdState,
    };

    use super::*;
    use crate::registry::Record;

    /// An app with its own registry.
    fn app() -> (App, Policy) {
        let mut app = App::new();
        app.init_resource::<Policy>()
            .add_observer(register_document)
            .add_observer(forget_document);
        let policy = app.world().resource::<Policy>().clone();
        (app, policy)
    }

    #[test]
    fn an_instance_records_the_host_that_composed_it() {
        let (mut app, policy) = app();
        let host_id = DocId([23; 32]);

        let host = app
            .world_mut()
            .spawn((Hsd::new(HsdState::new()), HsdDocId(host_id)))
            .id();
        let prim_id = PrimId::new();
        let prim = app.world_mut().spawn((Prim(prim_id), HsdChild(host))).id();

        let instance_id = DocId::instance(host_id, prim_id);
        app.world_mut().spawn((
            Hsd::new(HsdState::new()),
            HsdDocId(instance_id),
            ChildOf(prim),
        ));

        assert_eq!(policy.get(instance_id).host, Some(host_id));
        assert_eq!(
            policy.root(instance_id),
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
