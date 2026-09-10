use std::{
    collections::HashMap,
    sync::Weak,
    time::Duration,
};

use iroh_docs::{
    CapabilityKind,
    NamespaceId,
};

use crate::{
    Inner,
    Store,
};

/// Sweeps every `interval` until the store is dropped.
///
/// Holds the store weakly, so the sweep is what the store outlives rather than
/// the other way around.
pub async fn sweep_forever(store: Weak<Inner>, interval: Duration) {
    loop {
        n0_future::time::sleep(interval).await;

        let Some(store) = store.upgrade().map(Store) else {
            return;
        };

        if let Err(err) = sweep(&store).await {
            tracing::warn!(?err, "document retention sweep failed");
        }
    }
}

struct Candidate {
    ns:  NamespaceId,
    age: Duration,
}

async fn sweep(store: &Store) -> anyhow::Result<()> {
    let ages = store.visits().await?.into_iter().collect::<HashMap<_, _>>();
    let ttl = store.doc_ttl();
    let mut evictable = Vec::new();

    for (ns, capability) in store.list().await? {
        if matches!(capability, CapabilityKind::Write) {
            continue;
        }

        // An age of zero for a document with no visit on record, so the TTL
        // only ever takes something a visit was recorded for.
        let age = ages.get(&ns).copied().unwrap_or_default();
        if age >= ttl {
            evict(store, ns).await;
        } else {
            evictable.push(Candidate { ns, age });
        }
    }

    let Some(budget) = store.doc_budget() else {
        return Ok(());
    };
    trim_to_budget(store, evictable, budget).await
}

/// Measuring opens every document. The TTL pass runs first so its evictions
/// never wait behind a handle taken here.
async fn trim_to_budget(
    store: &Store,
    mut evictable: Vec<Candidate>,
    budget: u64,
) -> anyhow::Result<()> {
    let mut sizes = HashMap::new();
    let mut total = 0;

    for (ns, _) in store.list().await? {
        match measure(store, ns).await {
            // A document nothing can evict still counts against the budget.
            Ok(size) => {
                total += size;
                sizes.insert(ns, size);
            }
            Err(err) => {
                tracing::debug!(%ns, ?err, "leaving out a document that would not measure");
            }
        }
    }

    // Oldest visit first, so what goes is what has gone longest unused.
    evictable.sort_unstable_by_key(|candidate| std::cmp::Reverse(candidate.age));

    for candidate in evictable {
        if total <= budget {
            break;
        }
        if evict(store, candidate.ns).await {
            total = total.saturating_sub(sizes.get(&candidate.ns).copied().unwrap_or_default());
        }
    }

    Ok(())
}

/// The document is released before returning, so the eviction pass that follows
/// does not find every document held by the measurement.
async fn measure(store: &Store, ns: NamespaceId) -> anyhow::Result<u64> {
    store.open(ns).await?.size().await
}

/// A drop that fails because something still holds the replica is not an
/// error. The next sweep tries again.
async fn evict(store: &Store, ns: NamespaceId) -> bool {
    match store.drop(ns).await {
        Ok(()) => {
            tracing::info!(%ns, "evicted document");
            true
        }
        Err(err) => {
            tracing::debug!(%ns, ?err, "document is still held, leaving it");
            false
        }
    }
}
