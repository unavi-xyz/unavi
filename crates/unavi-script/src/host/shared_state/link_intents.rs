//! Scripts listening for portals that ask their space for a partner.

use std::{
    collections::{
        BTreeMap,
        HashSet,
    },
    sync::{
        Arc,
        atomic::{
            AtomicU64,
            Ordering,
        },
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

const UNCLAIMED: u64 = u64::MAX;

/// Who won an intent. Every subscriber offered it shares one, so at most one
/// script consumes it.
pub struct Claim(AtomicU64);

impl Claim {
    const fn new() -> Self {
        Self(AtomicU64::new(UNCLAIMED))
    }

    /// The subscription that won, if any.
    fn holder(&self) -> Option<u64> {
        match self.0.load(Ordering::Acquire) {
            UNCLAIMED => None,
            id => Some(id),
        }
    }

    fn take(&self, subscriber: u64) -> bool {
        self.0
            .compare_exchange(UNCLAIMED, subscriber, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

/// One intent as a subscriber receives it.
#[derive(Clone)]
pub struct Offer {
    pub intent: LinkIntent,
    claim:      Arc<Claim>,
    subscriber: u64,
}

impl Offer {
    /// Wins the intent for this subscription, unless it is already won.
    #[must_use]
    pub fn take(&self) -> bool {
        self.claim.take(self.subscriber)
    }
}

struct Subscriber {
    doc:          DocId,
    queue:        Arc<Queue<Offer>>,
    /// Queue overflows already answered.
    dropped_seen: u64,
    /// The claim of each intent this subscription was last offered.
    offered:      HashMap<LinkIntent, Arc<Claim>>,
}

/// Every open intent subscription, by the document of the script holding it.
#[derive(Resource, Clone, Default)]
pub struct LinkIntents(Arc<RwLock<Inner>>);

#[derive(Default)]
struct Inner {
    next:        u64,
    subscribers: HashMap<u64, Subscriber>,
    /// The claim of every intent offered and still derived.
    claims:      HashMap<LinkIntent, Arc<Claim>>,
}

impl LinkIntents {
    /// Opens a subscription for a script of `doc`, answering the id that
    /// closes it.
    pub fn open(&self, doc: DocId, queue: Arc<Queue<Offer>>) -> u64 {
        let mut inner = self.0.write();
        let id = inner.next;
        inner.next += 1;
        let dropped_seen = queue.dropped();
        inner.subscribers.insert(
            id,
            Subscriber {
                doc,
                queue,
                dropped_seen,
                offered: HashMap::default(),
            },
        );
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
            .map(|sub| sub.doc)
            .collect()
    }

    /// Brings every subscription in line with `derived`, the intents now
    /// raised and the documents each is addressed to, answering how many
    /// offers it queued.
    ///
    /// A subscription is offered each unclaimed intent addressed to its
    /// document once per claim, and again if its queue overflowed. An intent
    /// won by an open subscription is not offered again; one won by a closed
    /// subscription is released, and offered afresh. An intent no longer
    /// derived is forgotten, so it is asked for again if it returns.
    pub fn reconcile(&self, derived: &BTreeMap<LinkIntent, Vec<DocId>>) -> usize {
        let mut inner = self.0.write();
        let Inner {
            subscribers,
            claims,
            ..
        } = &mut *inner;

        claims.retain(|intent, claim| {
            derived.contains_key(intent)
                && claim
                    .holder()
                    .is_none_or(|holder| subscribers.contains_key(&holder))
        });

        for sub in subscribers.values_mut() {
            let dropped = sub.queue.dropped();
            if dropped != sub.dropped_seen {
                sub.dropped_seen = dropped;
                sub.offered.clear();
            }
            let doc = sub.doc;
            sub.offered.retain(|intent, offered| {
                derived.get(intent).is_some_and(|docs| docs.contains(&doc))
                    && claims
                        .get(intent)
                        .is_some_and(|claim| Arc::ptr_eq(claim, offered))
            });
        }

        let mut queued = 0;
        for (intent, docs) in derived {
            let claim = claims
                .entry(*intent)
                .or_insert_with(|| Arc::new(Claim::new()));
            if claim.holder().is_some() {
                continue;
            }
            for (id, sub) in subscribers.iter_mut() {
                if !docs.contains(&sub.doc) || sub.offered.contains_key(intent) {
                    continue;
                }
                sub.queue.push(Offer {
                    intent:     *intent,
                    claim:      Arc::clone(claim),
                    subscriber: *id,
                });
                sub.offered.insert(*intent, Arc::clone(claim));
                queued += 1;
            }
        }
        drop(inner);
        queued
    }
}
