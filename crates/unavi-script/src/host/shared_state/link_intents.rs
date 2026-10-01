//! Scripts listening for portals that ask their space for a partner.

use std::{
    collections::HashSet,
    sync::{
        Arc,
        atomic::AtomicBool,
    },
};

use bevy::{
    platform::collections::HashMap,
    prelude::Resource,
};
use hsd::{
    attributes::portal::LinkId,
    id::DocId,
};
use parking_lot::RwLock;

use crate::host::queue::Queue;

/// A portal in `source_space` bearing `link` that no portal in the receiving
/// space answers yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LinkIntent {
    pub source_space: DocId,
    pub link:         LinkId,
}

/// One intent as a subscriber receives it, with the claim every subscriber
/// offered it in the same round shares.
#[derive(Clone)]
pub struct Offer {
    pub intent: LinkIntent,
    pub claim:  Arc<AtomicBool>,
}

/// Every open intent subscription, by the document of the script holding it.
#[derive(Resource, Clone, Default)]
pub struct LinkIntents(Arc<RwLock<Inner>>);

#[derive(Default)]
struct Inner {
    next:        u64,
    subscribers: HashMap<u64, (DocId, Arc<Queue<Offer>>)>,
}

impl LinkIntents {
    /// Opens a subscription for a script of `doc`, answering the id that
    /// closes it.
    pub fn open(&self, doc: DocId, queue: Arc<Queue<Offer>>) -> u64 {
        let mut inner = self.0.write();
        let id = inner.next;
        inner.next += 1;
        inner.subscribers.insert(id, (doc, queue));
        id
    }

    pub fn close(&self, id: u64) {
        self.0.write().subscribers.remove(&id);
    }

    /// The documents with a script subscribed.
    #[must_use]
    pub fn listening(&self) -> HashSet<DocId> {
        self.0
            .read()
            .subscribers
            .values()
            .map(|(doc, _)| *doc)
            .collect()
    }

    /// Offers `intent` to every subscriber of `docs`, all sharing one claim.
    pub fn offer(&self, intent: LinkIntent, docs: &[DocId]) {
        let claim = Arc::new(AtomicBool::new(false));
        for (doc, queue) in self.0.read().subscribers.values() {
            if docs.contains(doc) {
                queue.push(Offer {
                    intent,
                    claim: Arc::clone(&claim),
                });
            }
        }
    }
}
