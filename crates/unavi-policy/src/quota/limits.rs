use std::collections::HashMap;

use crate::{
    quota::{
        Flow,
        Stock,
    },
    trust::Trust,
};

#[derive(Clone, Copy)]
pub struct FlowLimit {
    pub capacity:       f64,
    pub refill_per_sec: f64,
}

/// Per-scope caps. An absent entry means that resource is unbounded at this
/// scope, deferring entirely to ancestor caps.
#[derive(Default, Clone)]
pub struct Limits {
    pub stock: HashMap<Stock, u64>,
    pub flow:  HashMap<Flow, FlowLimit>,
}

impl Limits {
    /// Builds a scope's caps by asking for every variant in turn. Callers must
    /// answer with an exhaustive `match` and no wildcard, so a new [`Stock`] or
    /// [`Flow`] fails to compile until each scope has said what it costs there;
    /// an absent entry means unbounded.
    fn new(stock: impl Fn(Stock) -> Option<u64>, flow: impl Fn(Flow) -> Option<FlowLimit>) -> Self {
        Self {
            stock: Stock::ALL
                .into_iter()
                .filter_map(|s| stock(s).map(|cap| (s, cap)))
                .collect(),
            flow:  Flow::ALL
                .into_iter()
                .filter_map(|f| flow(f).map(|cap| (f, cap)))
                .collect(),
        }
    }
}

const MB: usize = 1024 * 1024;

/// Fraction of host RAM the combined wasm memory of every script may occupy.
const GLOBAL_WASM_MEMORY_PERCENT: u64 = 30;

// On wasm this reduces to a constant expression, which clippy flags as
// `const fn`-able; it can't be, since the non-wasm arm calls into `sysinfo`.
#[cfg_attr(target_family = "wasm", expect(clippy::missing_const_for_fn))]
fn host_total_memory() -> u64 {
    cfg_select! {
        // Not currently tracked on wasm, so this value isn't used.
        target_family = "wasm" => 2 * 1024 * MB as u64,
        _ => sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::nothing()
                .with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram()),
        )
        .total_memory(),
    }
}

impl Limits {
    #[must_use]
    pub fn global() -> Self {
        let budget = (host_total_memory() / 100).saturating_mul(GLOBAL_WASM_MEMORY_PERCENT);
        Self::new(
            |stock| match stock {
                Stock::WasmMemory => Some(budget),
                Stock::Documents
                | Stock::KvMemory
                | Stock::PortalWatches
                | Stock::Prims
                | Stock::Receptors
                | Stock::Slots => None,
            },
            |flow| match flow {
                Flow::BlobUpload
                | Flow::CreateDocument
                | Flow::CreatePrim
                | Flow::Emit
                | Flow::PortalOpen
                | Flow::SyncDoc => None,
            },
        )
    }

    #[must_use]
    pub fn document() -> Self {
        Self::new(
            |stock| match stock {
                Stock::KvMemory => Some(8 * MB as u64),
                Stock::WasmMemory => Some(128 * MB as u64),
                Stock::Documents => Some(256),
                Stock::Prims | Stock::Slots => Some(50_000),
                Stock::PortalWatches => Some(16),
                Stock::Receptors => Some(64),
            },
            |flow| match flow {
                Flow::CreateDocument => Some(FlowLimit {
                    capacity:       8.0,
                    refill_per_sec: 1.0,
                }),
                Flow::CreatePrim => Some(FlowLimit {
                    capacity:       2_000.0,
                    refill_per_sec: 500.0,
                }),
                Flow::Emit => Some(FlowLimit {
                    capacity:       256.0,
                    refill_per_sec: 16.0,
                }),
                Flow::BlobUpload => Some(FlowLimit {
                    capacity:       256.0,
                    refill_per_sec: 32.0,
                }),
                Flow::PortalOpen | Flow::SyncDoc => None,
            },
        )
    }

    #[must_use]
    pub fn space() -> Self {
        Self::new(
            |stock| match stock {
                Stock::KvMemory => Some(256 * MB as u64),
                Stock::WasmMemory => Some(512 * MB as u64),
                Stock::Documents => Some(4_000),
                Stock::Prims => Some(4_000_000),
                Stock::Slots => Some(8_000_000),
                Stock::PortalWatches => Some(128),
                Stock::Receptors => Some(32_000),
            },
            |flow| match flow {
                Flow::CreateDocument => Some(FlowLimit {
                    capacity:       256.0,
                    refill_per_sec: 16.0,
                }),
                Flow::CreatePrim => Some(FlowLimit {
                    capacity:       32_000.0,
                    refill_per_sec: 8_000.0,
                }),
                Flow::PortalOpen => Some(FlowLimit {
                    capacity:       8.0,
                    refill_per_sec: 1.0,
                }),
                Flow::Emit => Some(FlowLimit {
                    capacity:       4_096.0,
                    refill_per_sec: 2_048.0,
                }),
                Flow::SyncDoc => Some(FlowLimit {
                    capacity:       64.0,
                    refill_per_sec: 4.0,
                }),
                Flow::BlobUpload => Some(FlowLimit {
                    capacity:       4_096.0,
                    refill_per_sec: 256.0,
                }),
            },
        )
    }

    #[must_use]
    pub fn peer() -> Self {
        Self::new(
            |stock| match stock {
                Stock::KvMemory => Some(64 * MB as u64),
                Stock::WasmMemory => Some(256 * MB as u64),
                Stock::Documents => Some(1_000),
                Stock::Prims => Some(1_000_000),
                Stock::Slots => Some(2_000_000),
                Stock::PortalWatches => Some(32),
                Stock::Receptors => Some(2_000),
            },
            |flow| match flow {
                Flow::CreateDocument => Some(FlowLimit {
                    capacity:       64.0,
                    refill_per_sec: 8.0,
                }),
                Flow::CreatePrim => Some(FlowLimit {
                    capacity:       16_000.0,
                    refill_per_sec: 4_000.0,
                }),
                Flow::PortalOpen => Some(FlowLimit {
                    capacity:       4.0,
                    refill_per_sec: 0.5,
                }),
                Flow::Emit => Some(FlowLimit {
                    capacity:       2_048.0,
                    refill_per_sec: 1_024.0,
                }),
                Flow::SyncDoc => Some(FlowLimit {
                    capacity:       32.0,
                    refill_per_sec: 2.0,
                }),
                Flow::BlobUpload => Some(FlowLimit {
                    capacity:       2_048.0,
                    refill_per_sec: 128.0,
                }),
            },
        )
    }

    /// [`Self::peer`] scaled by the share a peer at `trust` may consume.
    #[must_use]
    pub fn for_trust(trust: Trust) -> Self {
        let mut limits = Self::peer();
        let share = match trust {
            // A blocked peer's content gets nothing at all, rather than a
            // small share: a zero-capacity bucket is refused on sight rather
            // than waited on.
            Trust::Blocked => 0.0,
            Trust::Guest => 0.25,
            Trust::Trusted | Trust::Myself => return limits,
        };

        for cap in limits.stock.values_mut() {
            *cap = (*cap as f64 * share) as u64;
        }
        for limit in limits.flow.values_mut() {
            limit.capacity *= share;
            limit.refill_per_sec *= share;
        }
        limits
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use crate::quota::{
        Flow,
        Quota,
        Reservation,
    };

    /// A shape mesh uploads POSITION, NORMAL, `UV_0`, and its index buffer:
    /// four blobs per mesh, each charged against [`Flow::BlobUpload`].
    const BLOBS_PER_MESH: usize = 4;

    #[test]
    fn document_blob_upload_sustains_an_init_burst() {
        let peer = Quota::root(Limits::peer());
        let doc = Quota::new(Limits::document(), Some(peer));
        for _ in 0..32 * BLOBS_PER_MESH {
            doc.spend(Flow::BlobUpload, 1.0)
                .expect("building init geometry must not exhaust the blob-upload quota");
        }
    }

    /// A UI surface pays three blobs per drawn body (`POSITION`, `NORMAL`,
    /// indices).
    #[test]
    fn one_document_cannot_afford_a_body_per_slot_up_front() {
        const BLOBS_PER_BODY: f64 = 3.0;
        let capacity = Limits::document()
            .flow
            .get(&Flow::BlobUpload)
            .expect("documents rate-limit blob uploads")
            .capacity;

        let bodies = capacity / BLOBS_PER_BODY;
        assert!(
            bodies < 100.0,
            "a document affords only {bodies} bodies in one burst, so a \
             surface that allocates every slot it might ever draw runs out \
             before it has drawn anything — build them as they are needed"
        );
    }

    #[test]
    fn global_caps_wasm_memory_at_a_fraction_of_host_ram() {
        let total = host_total_memory();
        assert!(total > 0, "host memory should be detectable in tests");
        let budget = *Limits::global()
            .stock
            .get(&Stock::WasmMemory)
            .expect("global caps wasm memory");
        assert_eq!(budget, (total / 100) * GLOBAL_WASM_MEMORY_PERCENT);
        assert!(budget < total, "scripts never get the whole host");
    }

    #[test]
    fn a_blocked_peer_gets_nothing_and_is_refused_immediately() {
        let quota = Quota::root(Limits::for_trust(Trust::Blocked));

        assert_eq!(
            quota.reserve(Flow::CreatePrim, 1.0),
            Reservation::Never,
            "a zero bucket never fills, so waiting on it would be a lie"
        );
        assert!(quota.try_charge(Stock::Prims, 1).is_err());
    }

    #[test]
    fn the_rungs_are_ordered_by_what_they_may_consume() {
        let prims = |trust| {
            *Limits::for_trust(trust)
                .stock
                .get(&Stock::Prims)
                .expect("peer limits cap prims")
        };

        assert!(prims(Trust::Blocked) < prims(Trust::Guest));
        assert!(prims(Trust::Guest) < prims(Trust::Trusted));
        assert_eq!(prims(Trust::Trusted), prims(Trust::Myself));
    }

    #[test]
    fn a_guest_still_gets_a_workable_budget() {
        let quota = Quota::root(Limits::for_trust(Trust::Guest));
        assert_eq!(
            quota.reserve(Flow::CreatePrim, 100.0),
            Reservation::Ready,
            "a first-time visitor's prop must build without waiting"
        );
    }
}
