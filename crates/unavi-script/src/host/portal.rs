//! `wired:portal`: portals between spaces, and travel through them.

use std::{
    collections::VecDeque,
    sync::Arc,
};

use hsd::{
    attributes::portal::{
        LinkId,
        PortalAttr,
        PortalDestination,
    },
    id::{
        DocId,
        PrimId,
    },
    property::{
        Payload,
        Property,
    },
};
use unavi_policy::{
    permissions::HostApi,
    quota::{
        Flow,
        Stock,
        StockLease,
    },
};
use unavi_space::membership::SpaceId;

use crate::{
    error::ScriptError,
    host::{
        ScriptHost,
        queue::Queue,
        scene::edit::{
            Layer,
            write_fields,
        },
        shared_state::{
            event_bus,
            link_intents::{
                LinkIntent,
                LinkIntents,
                Offer,
            },
        },
    },
};

/// Offers a subscription remembers for `claim`.
const MAX_OFFERS: usize = 64;

/// States `destination` on the portal of `prim` for the session, keeping the
/// portal's size.
async fn set_destination(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
    destination: PortalDestination,
) -> Result<(), ScriptError> {
    host.require(HostApi::Portal)?;
    let doc = host.owned_document(doc)?.clone();
    crate::quota::take(&host.quota, Flow::PortalOpen, 1)?;
    let mut portal = doc
        .read(|state| state.attribute::<PortalAttr>(prim).and_then(Result::ok))?
        .ok_or_else(|| ScriptError::invalid("the prim has no portal"))?;
    portal.destination = Some(destination);
    let value = portal.encode().map_err(ScriptError::internal)?;
    write_fields(
        host,
        &doc,
        Layer::Shared,
        prim,
        vec![(PortalAttr::NAME, Some(value))],
    )
    .await
}

pub async fn open(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
    target: DocId,
) -> Result<(), ScriptError> {
    let link = derive_link(host.document(doc)?.id, prim, target);
    set_destination(
        host,
        doc,
        prim,
        PortalDestination {
            space: target.0,
            link:  Some(link),
        },
    )
    .await
}

pub async fn pair(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
    intent: LinkIntent,
) -> Result<(), ScriptError> {
    set_destination(
        host,
        doc,
        prim,
        PortalDestination {
            space: intent.source_space.0,
            link:  Some(intent.link),
        },
    )
    .await
}

pub async fn travel(host: &ScriptHost, target: DocId) -> Result<(), ScriptError> {
    host.require(HostApi::Travel)?;
    crate::quota::take(&host.quota, Flow::PortalOpen, 1)?;
    host.world_call(move |world| {
        unavi_space::travel::request_travel(world, SpaceId::of_doc(target));
    })
    .await
}

/// The same on every peer that opens `prim` into `target`, since each runs the
/// opening script and a per-call id would leave the halves disagreeing.
fn derive_link(doc: DocId, prim: PrimId, target: DocId) -> LinkId {
    let mut hasher = blake3::Hasher::new_derive_key("unavi portal link");
    hasher.update(&doc.0);
    hasher.update(&prim.0);
    hasher.update(&target.0);
    let mut link = [0; 16];
    link.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    LinkId(link)
}

/// A guest's open intent subscription. Closes when dropped.
pub struct IntentSubscription {
    id:      u64,
    intents: LinkIntents,
    queue:   Arc<Queue<Offer>>,
    /// Offers drained so far, newest last, for `claim`.
    offers:  VecDeque<Offer>,
    _lease:  StockLease,
}

impl Drop for IntentSubscription {
    fn drop(&mut self) {
        self.intents.close(self.id);
    }
}

pub fn intents(host: &mut ScriptHost) -> Result<u32, ScriptError> {
    host.require(HostApi::Portal)?;
    let lease = host.quota.lease(Stock::Receptors, 1)?;
    let queue = Arc::new(Queue::default());
    let intents = host.shared.link_intents.clone();
    let id = intents.open(host.doc, Arc::clone(&queue));
    let subscription = IntentSubscription {
        id,
        intents,
        queue,
        offers: VecDeque::new(),
        _lease: lease,
    };
    Ok(host.intents.insert(subscription, &host.quota)?)
}

pub fn drain(
    host: &mut ScriptHost,
    subscription: u32,
    max: u32,
) -> Result<Vec<LinkIntent>, ScriptError> {
    let sub = host.intents.get_mut(subscription)?;
    let drained = sub.queue.drain(max);
    for offer in &drained {
        if sub.offers.len() >= MAX_OFFERS {
            sub.offers.pop_front();
        }
        sub.offers.push_back(offer.clone());
    }
    Ok(drained.into_iter().map(|offer| offer.intent).collect())
}

pub fn dropped(host: &ScriptHost, subscription: u32) -> Result<u64, ScriptError> {
    Ok(host.intents.get(subscription)?.queue.dropped())
}

/// Whether this subscription won `intent`, which it must have drained.
pub fn claim(
    host: &ScriptHost,
    subscription: u32,
    intent: LinkIntent,
) -> Result<bool, ScriptError> {
    let sub = host.intents.get(subscription)?;
    Ok(sub
        .offers
        .iter()
        .rev()
        .find(|offer| offer.intent == intent)
        .is_some_and(|offer| event_bus::claim(&offer.claim)))
}

pub fn close(host: &mut ScriptHost, subscription: u32) -> Result<(), ScriptError> {
    host.intents.remove(subscription).map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: DocId = DocId([1; 32]);
    const PRIM: PrimId = PrimId([2; 16]);

    #[test]
    fn a_link_is_stable_per_portal_and_target() {
        assert_eq!(
            derive_link(DOC, PRIM, DocId([3; 32])),
            derive_link(DOC, PRIM, DocId([3; 32]))
        );
        assert_ne!(
            derive_link(DOC, PRIM, DocId([3; 32])),
            derive_link(DOC, PRIM, DocId([4; 32]))
        );
        assert_ne!(
            derive_link(DOC, PRIM, DocId([3; 32])),
            derive_link(DOC, PrimId([5; 16]), DocId([3; 32]))
        );
    }
}
