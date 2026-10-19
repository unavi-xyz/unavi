//! Holds the documents of objects the local pointer is carrying.
//!
//! Grabbing an object this node does not author still moves it, since physics
//! does not care who holds the document. Only a claimable hold is taken: one
//! already tracked in state. The physics side neither knows nor needs to know
//! which of those two happened.

use bevy::{
    platform::collections::{
        HashMap,
        HashSet,
    },
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
use unavi_grab::held::Held;

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

/// The documents this node has taken a hold on for a held entity.
#[derive(Resource, Default)]
pub(crate) struct Holds(HashSet<DocId>);

/// Holds exactly the documents of held entities that this node may claim.
///
/// A document counts once it is tracked in state. An untracked one is
/// established by the publish path, and claiming it here would create presence
/// ahead of that upload. A hold is released when its entity is let go or
/// despawns.
pub(crate) fn reconcile_holds(
    carried: Query<Entity, With<Held>>,
    hsd_children: Query<&PrimOf>,
    docs: Docs,
    spaces: Query<&Space>,
    parents: Query<&ChildOf>,
    active_space: Option<Res<ActiveSpace>>,
    replicas: Res<Replicas>,
    view: Option<Res<SpaceView>>,
    mut holds: ResMut<Holds>,
) {
    let Some(view) = view else {
        return;
    };
    let active_space = active_space.and_then(|active| active.0);

    let mut wanted = HashMap::new();
    for entity in &carried {
        let Some((doc_entity, ns)) = resolve_doc(entity, &hsd_children, &docs) else {
            continue;
        };
        let Some(space) = resolve_space(doc_entity, &spaces, &parents).or_else(|| {
            let active = active_space?;
            spaces.get(active).ok().map(Space::id)
        }) else {
            continue;
        };
        let doc = DocId(*ns.as_bytes());
        if replicas.has_doc(space, doc) {
            wanted.insert(doc, space);
        }
    }

    for (&doc, &space) in &wanted {
        if holds.0.insert(doc) {
            info!(%doc, %space, "grab: taking hold of the object");
            view.local().take_hold(space, doc);
        }
    }
    holds.0.retain(|doc| {
        let keep = wanted.contains_key(doc);
        if !keep {
            view.local().release_hold(*doc);
        }
        keep
    });
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
