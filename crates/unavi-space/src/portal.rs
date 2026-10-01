//! Portals into spaces: loading each destination, and placing linked seams in
//! the space their document belongs to.

use bevy::{
    platform::collections::HashSet,
    prelude::*,
};
use bevy_hsd::{
    attributes::portal::PortalConfig,
    document::HsdDocId,
    prim::PrimOf,
};
use unavi_policy::{
    Policy,
    permissions::HostApi,
};
use unavi_portal::{
    SeamHome,
    SeamLink,
};

use crate::{
    authority::SpaceView,
    grid::ActiveSpace,
    index::{
        DocIndex,
        Index,
    },
    membership::{
        Space,
        SpaceId,
    },
    pinned::PinnedDoc,
};

/// Most spaces open at once through portals in peer-pinned documents.
pub const MAX_PEEKED_SPACES: usize = 8;

/// A space loaded only because a peer-pinned document has a portal to it.
///
/// It is fetched so the portal can show it, but it joins no gossip topic and
/// is never announced until the local agent actually enters it.
#[derive(Component)]
pub struct Peeked;

/// Loads the destination space of every portal. One in a document a peer
/// pinned opens [`Peeked`], within [`MAX_PEEKED_SPACES`], and only if its
/// author may open portals at all.
pub fn spawn_portal_space(
    trigger: On<Insert, PortalConfig>,
    portals: Query<(&PortalConfig, Option<&PrimOf>)>,
    docs: Query<(&HsdDocId, Has<PinnedDoc>)>,
    doc_index: Res<DocIndex>,
    spaces: Res<Index<Space>>,
    peeked: Query<(), With<Peeked>>,
    policy: Res<Policy>,
    view: Option<Res<SpaceView>>,
    // A `commands.spawn` isn't visible to `spaces` until the command queue
    // flushes, so a second portal to the same destination opened the same
    // frame wouldn't see the first's spawn without this.
    mut pending: Local<HashSet<SpaceId>>,
    mut commands: Commands,
) {
    let Ok((portal, prim_of)) = portals.get(trigger.entity) else {
        return;
    };
    let Some(dest) = &portal.0.destination else {
        return;
    };
    let id = SpaceId(dest.space);

    pending.retain(|pending_id| spaces.get(*pending_id).is_none());
    if spaces.get(id).is_some() || pending.contains(&id) {
        return;
    }

    let source = prim_of
        .and_then(|p| docs.get(p.0).ok())
        .map(|(doc, pinned)| {
            let root = policy.root(doc.0);
            let root_pinned = doc_index
                .get(root)
                .and_then(|e| docs.get(e).ok())
                .is_some_and(|(_, pinned)| pinned);
            (root, pinned || root_pinned)
        });
    let peek = source.is_some_and(|(_, pinned)| pinned);

    if peek {
        if peeked.iter().count() >= MAX_PEEKED_SPACES {
            debug!(space = %id, "peeked space cap reached; portal destination not loaded");
            return;
        }
        let allowed = source.is_some_and(|(root, _)| {
            view.as_deref()
                .is_some_and(|view| view.permissions(root).require(HostApi::Portal).is_ok())
        });
        if !allowed {
            return;
        }
    }

    pending.insert(id);
    let mut space = commands.spawn(Space(id));
    if peek {
        space.insert(Peeked);
    }
}

/// A peeked space the local agent walked into is entered like any other.
pub fn enter_peeked_space(
    active: Res<ActiveSpace>,
    peeked: Query<(), With<Peeked>>,
    mut commands: Commands,
) {
    if let Some(entity) = active.0
        && peeked.contains(entity)
    {
        commands.entity(entity).remove::<Peeked>();
    }
}

/// Keeps each linked seam's [`SeamHome`] on the space its document is in,
/// which is unknown until the document registers.
pub fn sync_seam_home(
    seams: Query<(Entity, &PrimOf, Option<&SeamHome>), With<SeamLink>>,
    docs: Query<&HsdDocId>,
    view: Option<Res<SpaceView>>,
    mut commands: Commands,
) {
    let Some(view) = view else {
        return;
    };
    for (seam, child, current) in &seams {
        let home = docs
            .get(child.0)
            .ok()
            .and_then(|doc| view.space_of(doc.0))
            .map(SpaceId::doc);
        match (home, current) {
            (Some(home), Some(cur)) if cur.0 == home => {}
            (Some(home), _) => {
                commands.entity(seam).insert(SeamHome(home));
            }
            (None, Some(_)) => {
                commands.entity(seam).remove::<SeamHome>();
            }
            (None, None) => {}
        }
    }
}
