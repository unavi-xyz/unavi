use std::collections::HashMap;

use crate::{
    Flow,
    Stock,
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

const KB: usize = 1024;
const MB: usize = 1024 * KB;

/// Largest payload a single `emit` may carry, fanned out to every receptor.
pub const MAX_EVENT_PAYLOAD_BYTES: usize = 64 * KB;

/// Largest string written into a synced document (names, relationship keys).
pub const MAX_NAME_BYTES: usize = KB;

/// Largest string a single text prim may carry.
///
/// The renderer bounds a string again at layout time, which is the backstop
/// for a document that arrived over the network rather than through a script.
/// This bound is the cheaper one: it stops the bytes being stored and synced
/// at all.
pub const MAX_TEXT_BYTES: usize = 4 * KB;

/// Largest vertex/index stream a single mesh write may upload.
pub const MAX_MESH_ELEMENTS: usize = 4 * MB;

/// Fraction of host RAM the combined wasm memory of every script may occupy.
const GLOBAL_WASM_MEMORY_PERCENT: u64 = 30;

// On wasm this reduces to a constant expression, which clippy flags as
// `const fn`-able; it can't be, since the non-wasm arm calls into `sysinfo`.
#[cfg_attr(target_family = "wasm", expect(clippy::missing_const_for_fn))]
fn host_total_memory() -> u64 {
    cfg_select! {
        // Not currently tracked on wasm, so this value isn't used.
        target_family = "wasm" => 2 * 1024 * MB as u64,
        _ => {
            sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::nothing().with_memory(sysinfo::MemoryRefreshKind::nothing().with_ram()),
            )
            .total_memory()
        }
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
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use crate::{
        Flow,
        Quota,
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
}
