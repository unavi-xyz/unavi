//! Takes and releases a document's hold in step with [`unavi_grab`]'s local
//! grab and release.
//!
//! Grabbing an object this node does not author still moves it — physics
//! does not care who holds the document — but only a claimable hold is
//! actually taken: one already tracked in state, with nobody else holding
//! it. The physics side neither knows nor needs to know which of those two
//! happened.

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::{
    document::{
        Hsd,
        HsdNamespace,
        Unplaced,
    },
    hierarchy::ancestors,
    prim::PrimOf,
};
use hsd::id::DocId;
use iroh_docs::NamespaceId;
use unavi_grab::events::{
    Grabbed,
    Released,
};

use crate::{
    authority::SpaceView,
    grid::ActiveSpace,
    membership::{
        Space,
        SpaceId,
    },
    replication::Replicas,
};

type Docs<'w, 's> = Query<'w, 's, &'static HsdNamespace, (With<Hsd>, Without<Unplaced>)>;

/// Which doc, if any, each currently-grabbed entity's hold was taken for.
///
/// `Released` can fire with its entity already despawned (a scene unload,
/// a script despawning what it grabbed), so release cannot re-resolve the
/// doc from the entity's components the way the grab did: it has to read
/// what the grab already recorded here instead.
#[derive(Resource, Default)]
pub(crate) struct TakenHolds(HashMap<Entity, DocId>);

/// Takes this node's hold on the grabbed entity's document, if it is one
/// this node may claim.
pub(crate) fn take_hold_on_grab(
    trigger: On<Grabbed>,
    hsd_children: Query<&PrimOf>,
    docs: Docs,
    spaces: Query<&Space>,
    parents: Query<&ChildOf>,
    active_space: Option<Res<ActiveSpace>>,
    replicas: Res<Replicas>,
    view: Option<Res<SpaceView>>,
    mut taken: ResMut<TakenHolds>,
) {
    let Some(view) = view else {
        debug!("grab: local peer id not initialized yet, skipping the hold");
        return;
    };

    let Some((doc_entity, doc_hash)) = resolve_doc(trigger.entity, &hsd_children, &docs) else {
        debug!(
            entity = %trigger.entity,
            "grab: grabbed entity has no HSD doc, skipping the hold",
        );
        return;
    };

    let active_space = active_space.and_then(|active| active.0);
    let space_hash = resolve_space(doc_entity, &spaces, &parents).or_else(|| {
        let active = active_space?;
        spaces.get(active).ok().map(Space::id)
    });

    let Some(space_hash) = space_hash else {
        warn!(
            doc = %doc_hash,
            "grab: no enclosing space and no active space, skipping the hold",
        );
        return;
    };

    let (space, doc) = (space_hash, DocId(*doc_hash.as_bytes()));

    // Only take hold of a doc already tracked in state. An untracked
    // doc is established by the publish path; claiming here would create
    // presence ahead of that upload.
    if !replicas.has_doc(space, doc) {
        debug!(doc = %doc_hash, "grab: doc not tracked in state, skipping the hold");
        return;
    }

    info!(doc = %doc_hash, space = %space_hash, "grab: taking hold of the object");
    view.local().take_hold(space, doc);
    taken.0.insert(trigger.entity, doc);
}

/// Drops this node's hold on the released entity's document, if its grab
/// took one.
///
/// Reads [`TakenHolds`] rather than the entity's own components: a release
/// triggered by a despawn runs with the rest of the entity already gone.
pub(crate) fn release_hold_on_release(
    trigger: On<Released>,
    view: Option<Res<SpaceView>>,
    mut taken: ResMut<TakenHolds>,
) {
    let Some(doc) = taken.0.remove(&trigger.entity) else {
        return;
    };
    let Some(view) = view else {
        return;
    };
    view.local().release_hold(doc, None);
}

fn resolve_doc(
    entity: Entity,
    hsd_children: &Query<&PrimOf>,
    docs: &Docs,
) -> Option<(Entity, NamespaceId)> {
    if let Ok(record) = docs.get(entity) {
        return Some((entity, record.0.id()));
    }
    let child = hsd_children.get(entity).ok()?;
    let record = docs.get(child.0).ok()?;
    Some((child.0, record.0.id()))
}

fn resolve_space(
    doc_entity: Entity,
    spaces: &Query<&Space>,
    parents: &Query<&ChildOf>,
) -> Option<SpaceId> {
    ancestors(doc_entity, parents).find_map(|at| spaces.get(at).ok().map(Space::id))
}

#[cfg(test)] mod tests;
