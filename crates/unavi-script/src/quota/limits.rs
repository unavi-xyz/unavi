use std::collections::HashMap;

use crate::quota::{
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

const KB: usize = 1024;
const MB: usize = 1024 * KB;
const GB: usize = 1024 * MB;

/// Largest payload a single `emit` may carry, fanned out to every receptor.
pub const MAX_EVENT_PAYLOAD_BYTES: usize = 64 * KB;

/// Largest string written into a synced document (names, relationship keys).
pub const MAX_NAME_BYTES: usize = KB;

/// Largest vertex/index stream a single mesh write may upload.
pub const MAX_MESH_ELEMENTS: usize = 4 * MB;

/// Fraction of host RAM the combined wasm memory of every script may occupy.
const GLOBAL_WASM_MEMORY_PERCENT: u64 = 30;

fn host_total_memory() -> u64 {
    cfg_select! {
        // WasmMemory is not currently tracked on wasm, so this value isn't used.
        target_family = "wasm" =>  2 * GB,
        _ => {
            use sysinfo::{
                MemoryRefreshKind,
                RefreshKind,
                System,
            };
            System::new_with_specifics(
                RefreshKind::nothing().with_memory(MemoryRefreshKind::nothing().with_ram()),
            )
            .total_memory()
        }
    }
}

impl Limits {
    #[must_use]
    pub fn global() -> Self {
        let budget = (host_total_memory() / 100).saturating_mul(GLOBAL_WASM_MEMORY_PERCENT);
        let stock = HashMap::from([(Stock::WasmMemory, budget)]);
        Self {
            stock,
            flow: HashMap::new(),
        }
    }

    #[must_use]
    pub fn document() -> Self {
        let stock = HashMap::from([
            (Stock::WasmMemory, (256 * MB) as u64),
            (Stock::Documents, 256),
            (Stock::Prims, 50_000),
            (Stock::Slots, 50_000),
            (Stock::PortalWatches, 16),
            (Stock::Receptors, 64),
        ]);
        let flow = HashMap::from([
            (
                Flow::CreateDocument,
                FlowLimit {
                    capacity:       8.0,
                    refill_per_sec: 1.0,
                },
            ),
            (
                Flow::CreatePrim,
                FlowLimit {
                    capacity:       2_000.0,
                    refill_per_sec: 500.0,
                },
            ),
            (
                Flow::Emit,
                FlowLimit {
                    capacity:       256.0,
                    refill_per_sec: 16.0,
                },
            ),
            (
                Flow::BlobUpload,
                FlowLimit {
                    capacity:       256.0,
                    refill_per_sec: 32.0,
                },
            ),
        ]);
        Self { stock, flow }
    }

    #[must_use]
    pub fn space() -> Self {
        let stock = HashMap::from([
            (Stock::WasmMemory, GB as u64),
            (Stock::Documents, 4_000),
            (Stock::Prims, 4_000_000),
            (Stock::Slots, 8_000_000),
            (Stock::PortalWatches, 128),
            (Stock::Receptors, 32_000),
        ]);
        let flow = HashMap::from([
            (
                Flow::CreateDocument,
                FlowLimit {
                    capacity:       256.0,
                    refill_per_sec: 16.0,
                },
            ),
            (
                Flow::CreatePrim,
                FlowLimit {
                    capacity:       32_000.0,
                    refill_per_sec: 8_000.0,
                },
            ),
            (
                Flow::PortalOpen,
                FlowLimit {
                    capacity:       8.0,
                    refill_per_sec: 1.0,
                },
            ),
            (
                Flow::Emit,
                FlowLimit {
                    capacity:       4_096.0,
                    refill_per_sec: 2_048.0,
                },
            ),
            (
                Flow::Publish,
                FlowLimit {
                    capacity:       64.0,
                    refill_per_sec: 4.0,
                },
            ),
            (
                Flow::BlobUpload,
                FlowLimit {
                    capacity:       4_096.0,
                    refill_per_sec: 256.0,
                },
            ),
        ]);
        Self { stock, flow }
    }

    #[must_use]
    pub fn peer() -> Self {
        let stock = HashMap::from([
            (Stock::WasmMemory, GB as u64),
            (Stock::Documents, 1_000),
            (Stock::Prims, 1_000_000),
            (Stock::Slots, 2_000_000),
            (Stock::PortalWatches, 32),
            (Stock::Receptors, 2_000),
        ]);
        let flow = HashMap::from([
            (
                Flow::CreateDocument,
                FlowLimit {
                    capacity:       64.0,
                    refill_per_sec: 8.0,
                },
            ),
            (
                Flow::CreatePrim,
                FlowLimit {
                    capacity:       16_000.0,
                    refill_per_sec: 4_000.0,
                },
            ),
            (
                Flow::PortalOpen,
                FlowLimit {
                    capacity:       4.0,
                    refill_per_sec: 0.5,
                },
            ),
            (
                Flow::Emit,
                FlowLimit {
                    capacity:       2_048.0,
                    refill_per_sec: 1_024.0,
                },
            ),
            (
                Flow::Publish,
                FlowLimit {
                    capacity:       32.0,
                    refill_per_sec: 2.0,
                },
            ),
            (
                Flow::BlobUpload,
                FlowLimit {
                    capacity:       2_048.0,
                    refill_per_sec: 128.0,
                },
            ),
        ]);
        Self { stock, flow }
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::*;
    use crate::quota::{
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
