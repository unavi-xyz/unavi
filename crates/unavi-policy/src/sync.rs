use bevy::prelude::*;
use bevy_hsd::{
    HsdChild,
    HsdDocId,
};
use hsd::id::DocId;

use crate::{
    owner::Owner,
    permissions::Permissions,
    registry::Policy,
};

/// The components stating one document's policy. Each arrives on its own
/// schedule, so every trigger re-reads all of them.
type DocumentQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static HsdDocId,
        Option<&'static Permissions>,
        Option<&'static Owner>,
        Option<&'static ChildOf>,
    ),
>;

/// Re-reads a document's entity into the registry.
pub fn sync_on<C: Component>(
    trigger: On<Insert, C>,
    docs: DocumentQuery,
    prims: Query<&HsdChild>,
    ids: Query<&HsdDocId>,
    policy: Res<Policy>,
) {
    let Ok((doc, permissions, owner, parent)) = docs.get(trigger.entity) else {
        return;
    };
    let host = host_of(parent, &prims, &ids);
    let inherited = host.map(|host| policy.get(host).permissions);

    policy.update(doc.0, |record| {
        // A prefab instance runs with the permissions of the document that
        // composed it in. Anything else states its own.
        if let Some(permissions) = permissions.copied().or(inherited) {
            record.permissions = permissions;
        }
        if let Some(owner) = owner {
            record.owner = Some(*owner);
        }
        record.host = host;
    });
}

/// The document that composed this one in, for a prefab instance.
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
        state::SceneState,
    };

    use super::*;
    use crate::permissions::ApiName;

    /// An app with its own registry.
    fn app() -> (App, Policy) {
        let mut app = App::new();
        app.init_resource::<Policy>()
            .add_observer(sync_on::<HsdDocId>)
            .add_observer(sync_on::<Permissions>)
            .add_observer(sync_on::<Owner>)
            .add_observer(forget_document);
        let policy = app.world().resource::<Policy>().clone();
        (app, policy)
    }

    #[test]
    fn a_document_stated_before_it_had_an_id_still_registers() {
        let (mut app, policy) = app();
        let id = DocId([21; 32]);

        let entity = app
            .world_mut()
            .spawn((Permissions::system(), Owner::System))
            .id();
        assert_eq!(
            policy.get(id).permissions,
            Permissions::untrusted(),
            "nothing keys the record until the document has an id"
        );

        app.world_mut()
            .entity_mut(entity)
            .insert((Hsd::new(SceneState::new()), HsdDocId(id)));

        assert_eq!(policy.get(id).permissions, Permissions::system());
        assert_eq!(policy.get(id).owner, Some(Owner::System));

        app.world_mut().entity_mut(entity).despawn();
        assert_eq!(
            policy.get(id).permissions,
            Permissions::untrusted(),
            "a despawned document must not leave its grant behind"
        );
    }

    #[test]
    fn a_grant_stated_after_registration_takes_effect() {
        let (mut app, policy) = app();
        let id = DocId([22; 32]);

        let entity = app
            .world_mut()
            .spawn((Hsd::new(SceneState::new()), HsdDocId(id)))
            .id();
        assert_eq!(policy.get(id).permissions, Permissions::untrusted());

        app.world_mut()
            .entity_mut(entity)
            .insert(Permissions::space());
        assert_eq!(
            policy.get(id).permissions,
            Permissions::space(),
            "a document that changes its mind must not be ignored"
        );
    }

    #[test]
    fn an_instance_records_its_host_and_inherits_its_permissions() {
        let (mut app, policy) = app();
        let host_id = DocId([23; 32]);

        let host = app
            .world_mut()
            .spawn((
                Hsd::new(SceneState::new()),
                HsdDocId(host_id),
                Permissions::space(),
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
        assert!(
            policy
                .get(instance_id)
                .permissions
                .require(ApiName::Identity)
                .is_ok(),
            "an instance runs with the space's permissions, not a peer's"
        );
        assert_eq!(
            policy.get(instance_id).owner,
            None,
            "an instance states no owner of its own; the host chain answers"
        );
    }
}
