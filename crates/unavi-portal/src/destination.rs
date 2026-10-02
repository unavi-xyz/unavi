//! Where a portal opens onto: another portal across a link, or a document
//! root.

use std::collections::BTreeMap;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::{
    document::{
        Hsd,
        HsdDocId,
        Unplaced,
    },
    prim::{
        Prim,
        PrimOf,
    },
};
use hsd::{
    attributes::portal::LinkId,
    id::{
        DocId,
        PrimId,
    },
};

use crate::portal::{
    PortalHome,
    PortalLink,
    PortalTargetDoc,
};

#[cfg(test)] mod tests;

/// The entity a portal currently opens onto: its link partner, or its
/// target space's root document when it has none.
#[derive(Component)]
#[relationship(relationship_target = Arrivals)]
pub struct Destination(pub Entity);

/// Portals (or nothing, for a document root) whose [`Destination`] points
/// here.
#[derive(Component, Default)]
#[relationship_target(relationship = Destination)]
pub struct Arrivals(Vec<Entity>);

/// A portal carrying a link, as [`pair_links`] sees it.
#[derive(Clone, Copy, Debug)]
pub struct LinkedPortal {
    pub entity: Entity,
    /// The portal's document and prim, which order candidates identically
    /// on every client.
    pub key:    (DocId, PrimId),
    pub home:   DocId,
    pub target: DocId,
    pub link:   LinkId,
}

/// Each linked portal's partner: the lowest-keyed other portal standing in
/// its target space, bearing its link, and aimed back at its home.
#[must_use]
pub fn pair_links(portals: &[LinkedPortal]) -> HashMap<Entity, Entity> {
    let mut by_home = BTreeMap::<(DocId, LinkId), Vec<&LinkedPortal>>::new();
    for portal in portals {
        by_home
            .entry((portal.home, portal.link))
            .or_default()
            .push(portal);
    }

    portals
        .iter()
        .filter_map(|portal| {
            let partner = by_home
                .get(&(portal.target, portal.link))?
                .iter()
                .filter(|far| far.target == portal.home)
                .filter(|far| far.entity != portal.entity)
                .min_by_key(|far| far.key)?;
            Some((portal.entity, partner.entity))
        })
        .collect()
}

/// Glues a paired portal to its partner, and any other portal to its target
/// space's root document.
pub fn resolve_destinations(
    portals: Query<(
        Entity,
        &PortalTargetDoc,
        Option<&PortalLink>,
        Option<&PortalHome>,
        Option<&Prim>,
        Option<&PrimOf>,
        Option<&Destination>,
    )>,
    docs: Query<(Entity, &HsdDocId), (With<Hsd>, Without<Unplaced>)>,
    mut commands: Commands,
) {
    let roots: HashMap<DocId, Entity> = docs.iter().map(|(e, id)| (id.0, e)).collect();

    let linked = portals
        .iter()
        .filter_map(|(entity, target, link, home, prim, child, _)| {
            let (_, doc) = docs.get(child?.0).ok()?;
            Some(LinkedPortal {
                entity,
                key: (doc.0, prim?.0),
                home: home?.0,
                target: target.0,
                link: link?.0,
            })
        })
        .collect::<Vec<_>>();
    let partners = pair_links(&linked);

    for (portal, target, .., current) in &portals {
        let resolved = partners
            .get(&portal)
            .or_else(|| roots.get(&target.0))
            .copied();
        reconcile(portal, resolved, current, &mut commands);
    }
}

fn reconcile(
    portal: Entity,
    resolved: Option<Entity>,
    current: Option<&Destination>,
    commands: &mut Commands,
) {
    match (resolved, current) {
        (Some(e), Some(cur)) if cur.0 == e => {}
        (Some(e), _) => {
            commands.entity(portal).insert(Destination(e));
        }
        (None, Some(_)) => {
            commands.entity(portal).remove::<Destination>();
        }
        (None, None) => {}
    }
}
