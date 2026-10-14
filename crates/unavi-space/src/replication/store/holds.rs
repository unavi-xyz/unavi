//! Holds: which peer simulates a document's rigid bodies.
//!
//! A hold carries its holder's stamp, and the highest `(at, endpoint)`
//! resolves the holder, so every peer holding the same holds names the same one
//! whatever order they arrived in. A receiver refuses a stamp past its clock's
//! skew allowance, which bounds how long a forged one outranks every grab.

use std::collections::HashMap;

use hsd::id::DocId;
use iroh::EndpointId;

use crate::{
    membership::SpaceId,
    replication::message::HoldClaim,
};

/// Most documents one peer may hold at once.
pub const MAX_HOLDS_PER_PEER: usize = 256;

struct Hold {
    space: SpaceId,
    at:    u64,
}

#[derive(Default)]
pub(super) struct Holds {
    by_doc: HashMap<DocId, HashMap<EndpointId, Hold>>,
    counts: HashMap<EndpointId, usize>,
}

impl Holds {
    /// The peer whose hold on `doc` in `space` ranks highest.
    pub(super) fn holder(&self, doc: DocId, space: SpaceId) -> Option<EndpointId> {
        self.by_doc
            .get(&doc)?
            .iter()
            .filter(|(_, hold)| hold.space == space)
            .max_by_key(|(peer, hold)| (hold.at, **peer))
            .map(|(peer, _)| *peer)
    }

    /// The newest stamp held on `doc`.
    pub(super) fn newest(&self, doc: DocId) -> Option<u64> {
        self.by_doc.get(&doc)?.values().map(|hold| hold.at).max()
    }

    /// Every hold `peer` states, as `(doc, space, at)`.
    pub(super) fn of_peer(&self, peer: EndpointId) -> Vec<(DocId, SpaceId, u64)> {
        self.by_doc
            .iter()
            .filter_map(|(doc, holds)| {
                let hold = holds.get(&peer)?;
                Some((*doc, hold.space, hold.at))
            })
            .collect()
    }

    /// Every peer with at least one hold.
    pub(super) fn peers(&self) -> impl Iterator<Item = EndpointId> + '_ {
        self.counts.keys().copied()
    }

    /// Adds or restates `peer`'s hold. `false` if `peer` already holds
    /// [`MAX_HOLDS_PER_PEER`] other documents.
    pub(super) fn add(&mut self, peer: EndpointId, hold: HoldClaim) -> bool {
        let HoldClaim { doc, space, at } = hold;
        let is_new = self
            .by_doc
            .get(&doc)
            .is_none_or(|holds| !holds.contains_key(&peer));
        if is_new {
            let count = self.counts.entry(peer).or_default();
            if *count >= MAX_HOLDS_PER_PEER {
                return false;
            }
            *count += 1;
        }
        self.by_doc
            .entry(doc)
            .or_default()
            .insert(peer, Hold { space, at });
        true
    }

    pub(super) fn remove(&mut self, peer: EndpointId, doc: DocId) {
        let Some(holds) = self.by_doc.get_mut(&doc) else {
            return;
        };
        if holds.remove(&peer).is_none() {
            return;
        }
        if holds.is_empty() {
            self.by_doc.remove(&doc);
        }
        if let Some(count) = self.counts.get_mut(&peer) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.counts.remove(&peer);
            }
        }
    }
}
