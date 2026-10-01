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
use unavi_portal_protocol::{
    INTENT_CHANNEL,
    LinkIntent,
};
use unavi_space::view::SpaceView;

use crate::{
    engine::InitializedScript,
    runtime::shared::registry::event::EventBus,
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

/// Every (intent, receiving document) the loaded portals ask for: a link with
/// no portal in its target space bearing it, addressed to each document in
/// that space.
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
            source_space: portal.home.0,
            link:         portal.link.0,
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

/// Emits each derived intent once per receiving document, as soon as that
/// document's scripts are listening. A pair is forgotten once no portal
/// derives it, so a far half that disappears is asked for again.
pub fn emit_link_intents(
    portals: Query<(&PortalConfig, &PrimOf)>,
    docs: Query<(Entity, &HsdDocId)>,
    ready: Query<&PrimOf, With<InitializedScript>>,
    view: Option<Res<SpaceView>>,
    event_bus: Res<EventBus>,
    mut delivered: Local<HashSet<(LinkIntent, DocId)>>,
) {
    let Some(view) = view else {
        return;
    };

    let linked = portals
        .iter()
        .filter_map(|(cfg, child)| {
            let dest = cfg.0.destination?;
            let (_, doc) = docs.get(child.0).ok()?;
            Some(LinkedPortal {
                home:   view.space_of(doc.0)?,
                target: DocId(dest.space),
                link:   dest.link?,
            })
        })
        .collect::<Vec<_>>();
    if linked.is_empty() {
        delivered.clear();
        return;
    }

    let receivers = docs
        .iter()
        .filter_map(|(_, doc)| {
            Some(IntentReceiver {
                doc:   doc.0,
                space: view.space_of(doc.0)?,
            })
        })
        .collect::<Vec<_>>();
    let listening = ready
        .iter()
        .filter_map(|c| docs.get(c.0).ok())
        .map(|(_, doc)| doc.0)
        .filter(|doc| event_bus.doc_has_receptor(*doc, INTENT_CHANNEL))
        .collect::<HashSet<_>>();

    let intents = derive_intents(&linked, &receivers);
    delivered.retain(|(intent, doc)| intents.get(intent).is_some_and(|d| d.contains(doc)));

    for (intent, targets) in intents {
        let due = targets
            .into_iter()
            .filter(|doc| listening.contains(doc) && !delivered.contains(&(intent, *doc)))
            .collect::<Vec<_>>();
        if due.is_empty() {
            continue;
        }
        match postcard::to_allocvec(&intent) {
            Ok(bytes) => event_bus.emit_from_host(&due, INTENT_CHANNEL, bytes),
            Err(err) => {
                warn!(?err, "encode link intent");
                continue;
            }
        }
        delivered.extend(due.into_iter().map(|doc| (intent, doc)));
    }
}
