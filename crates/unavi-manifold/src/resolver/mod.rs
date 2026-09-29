use std::collections::BTreeMap;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdDocId,
    Prim,
};
use hsd::{
    attributes::portal::LinkId,
    id::{
        DocId,
        PrimId,
    },
};

use crate::{
    GluedTo,
    SeamHome,
    SeamLink,
    SeamTargetDoc,
};

#[cfg(test)] mod tests;

/// A seam carrying a link, as [`pair_links`] sees it.
#[derive(Clone, Copy, Debug)]
pub struct LinkedSeam {
    pub entity: Entity,
    /// The seam's document and prim, which order candidates identically on
    /// every client.
    pub key:    (DocId, PrimId),
    pub home:   DocId,
    pub target: DocId,
    pub link:   LinkId,
}

/// Each linked seam's partner: the lowest-keyed other seam standing in its
/// target space, bearing its link, and aimed back at its home.
#[must_use]
pub fn pair_links(seams: &[LinkedSeam]) -> HashMap<Entity, Entity> {
    let mut by_home = BTreeMap::<(DocId, LinkId), Vec<&LinkedSeam>>::new();
    for seam in seams {
        by_home
            .entry((seam.home, seam.link))
            .or_default()
            .push(seam);
    }

    seams
        .iter()
        .filter_map(|seam| {
            let partner = by_home
                .get(&(seam.target, seam.link))?
                .iter()
                .filter(|far| far.target == seam.home)
                .filter(|far| far.entity != seam.entity)
                .min_by_key(|far| far.key)?;
            Some((seam.entity, partner.entity))
        })
        .collect()
}

/// Glues a paired seam to its partner, and any other seam to its target
/// space's root document.
pub fn resolve_seams(
    seams: Query<(
        Entity,
        &SeamTargetDoc,
        Option<&SeamLink>,
        Option<&SeamHome>,
        Option<&Prim>,
        Option<&HsdChild>,
        Option<&GluedTo>,
    )>,
    docs: Query<(Entity, &HsdDocId), With<Hsd>>,
    mut commands: Commands,
) {
    let roots: HashMap<DocId, Entity> = docs.iter().map(|(e, id)| (id.0, e)).collect();

    let linked = seams
        .iter()
        .filter_map(|(entity, target, link, home, prim, child, _)| {
            let (_, doc) = docs.get(child?.0).ok()?;
            Some(LinkedSeam {
                entity,
                key: (doc.0, prim?.0),
                home: home?.0,
                target: target.0,
                link: link?.0,
            })
        })
        .collect::<Vec<_>>();
    let partners = pair_links(&linked);

    for (seam, target, .., current) in &seams {
        let resolved = partners
            .get(&seam)
            .or_else(|| roots.get(&target.0))
            .copied();
        reconcile(seam, resolved, current, &mut commands);
    }
}

fn reconcile(
    seam: Entity,
    resolved: Option<Entity>,
    current: Option<&GluedTo>,
    commands: &mut Commands,
) {
    match (resolved, current) {
        (Some(e), Some(cur)) if cur.0 == e => {}
        (Some(e), _) => {
            commands.entity(seam).insert(GluedTo(e));
        }
        (None, Some(_)) => {
            commands.entity(seam).remove::<GluedTo>();
        }
        (None, None) => {}
    }
}
