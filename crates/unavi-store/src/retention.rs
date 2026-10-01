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
    Store,
    StoreInner,
};

/// Sweeps every `interval` until the store drops. The store is held weakly, so
/// the task never keeps it alive.
pub async fn sweep_forever(store: Weak<StoreInner>, interval: Duration) {
    loop {
        n0_future::time::sleep(interval).await;

        let Some(store) = store.upgrade().map(Store) else {
            return;
        };

        if let Err(err) = store.sweep().await {
            tracing::warn!(?err, "document retention sweep failed");
        }
    }
}

struct Candidate {
    ns:  NamespaceId,
    age: Duration,
}

impl Store {
    async fn sweep(&self) -> anyhow::Result<()> {
        let ages = self.visits().await?;
        let mut evictable = Vec::new();

        for (ns, capability) in self.list().await? {
            if matches!(capability, CapabilityKind::Write) {
                continue;
            }

            // A document with no visit reads as age zero, so the TTL only
            // takes documents a visit was recorded for.
            let age = ages.get(&ns).copied().unwrap_or_default();
            if age >= self.0.doc_ttl {
                self.evict(ns).await;
            } else {
                evictable.push(Candidate { ns, age });
            }
        }

        let Some(budget) = self.0.doc_budget else {
            return Ok(());
        };
        self.trim_to_budget(evictable, budget).await
    }

    /// Opens every document to measure it, so the TTL pass runs first and its
    /// evictions never wait behind a handle taken here.
    async fn trim_to_budget(
        &self,
        mut evictable: Vec<Candidate>,
        budget: u64,
    ) -> anyhow::Result<()> {
        let mut sizes = HashMap::new();
        let mut total = 0;

        // Listed again, so what the TTL pass evicted is out of the total.
        for (ns, _) in self.list().await? {
            match self.measure(ns).await {
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
            if self.evict(candidate.ns).await {
                total = total.saturating_sub(sizes.get(&candidate.ns).copied().unwrap_or_default());
            }
        }

        Ok(())
    }

    /// Opens and releases the document, so the eviction pass that follows does
    /// not find every document held by the measurement.
    async fn measure(&self, ns: NamespaceId) -> anyhow::Result<u64> {
        self.open(ns).await?.size().await
    }

    /// A failed remove leaves the document for the next sweep.
    async fn evict(&self, ns: NamespaceId) -> bool {
        match self.remove(ns).await {
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
}
