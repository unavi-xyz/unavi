//! Finds portals whose far half is missing, and offers each to the scripts of
//! the space it points into.

use std::collections::{
    BTreeMap,
    HashSet,
};

use bevy::prelude::*;
use bevy_hsd::{
    attributes::portal::PortalConfig,
    document::HsdDocId,
    prim::PrimOf,
};
use hsd::{
    attributes::portal::LinkId,
    id::DocId,
};
use unavi_space::authority::SpaceView;

use crate::host::shared_state::link_intents::{
    LinkIntent,
    LinkIntents,
};

#[cfg(test)] mod tests;

/// A loaded portal carrying a link, placed in the space its document is in.
#[derive(Clone, Copy, Debug)]
pub struct LinkedPortal {
    pub home:   DocId,
    pub target: DocId,
    pub link:   LinkId,
}

/// A loaded document, placed in the space it is in.
#[derive(Clone, Copy, Debug)]
pub struct IntentReceiver {
    pub doc:   DocId,
    pub space: DocId,
}

/// Every intent the loaded portals raise, with the documents it is addressed
/// to: a link with no portal in its target space bearing it, addressed to
/// each document in that space.
#[must_use]
pub fn derive_intents(
    portals: &[LinkedPortal],
    docs: &[IntentReceiver],
) -> BTreeMap<LinkIntent, Vec<DocId>> {
    let answered: HashSet<(DocId, LinkId)> = portals.iter().map(|p| (p.home, p.link)).collect();

    let mut intents = BTreeMap::<LinkIntent, Vec<DocId>>::new();
    for portal in portals {
        if answered.contains(&(portal.target, portal.link)) {
            continue;
        }
        let intent = LinkIntent {
            source_space: portal.home,
            link:         portal.link,
        };
        for receiver in docs.iter().filter(|d| d.space == portal.target) {
            let targets = intents.entry(intent).or_default();
            if !targets.contains(&receiver.doc) {
                targets.push(receiver.doc);
            }
        }
    }
    intents
}

/// Offers each derived intent once per receiving document, as soon as a script
/// of that document subscribes. A pair is forgotten once no portal raises it,
/// so a far half that disappears is asked for again.
pub fn offer_link_intents(
    portals: Query<(&PortalConfig, &PrimOf)>,
    docs: Query<&HsdDocId>,
    view: Option<Res<SpaceView>>,
    intents: Res<LinkIntents>,
    mut offered: Local<HashSet<(LinkIntent, DocId)>>,
) {
    let Some(view) = view else {
        return;
    };

    let linked = portals
        .iter()
        .filter_map(|(cfg, prim_of)| {
            let dest = cfg.0.destination?;
            let doc = docs.get(prim_of.0).ok()?;
            Some(LinkedPortal {
                home:   view.space_of(doc.0)?.doc(),
                target: DocId(dest.space),
                link:   dest.link?,
            })
        })
        .collect::<Vec<_>>();
    if linked.is_empty() {
        offered.clear();
        return;
    }

    let listening = intents.listening();
    let receivers = listening
        .iter()
        .filter_map(|doc| {
            Some(IntentReceiver {
                doc:   *doc,
                space: view.space_of(*doc)?.doc(),
            })
        })
        .collect::<Vec<_>>();

    let derived = derive_intents(&linked, &receivers);
    offered.retain(|(intent, doc)| derived.get(intent).is_some_and(|d| d.contains(doc)));

    for (intent, targets) in derived {
        let due = targets
            .into_iter()
            .filter(|doc| !offered.contains(&(intent, *doc)))
            .collect::<Vec<_>>();
        if due.is_empty() {
            continue;
        }
        intents.offer(intent, &due);
        offered.extend(due.into_iter().map(|doc| (intent, doc)));
    }
}
