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

const MIB: u64 = 1024 * 1024;

/// Largest payload a single `emit` may carry, fanned out to every receptor.
pub const MAX_EVENT_PAYLOAD_BYTES: usize = 64 * 1024;

/// Largest string written into a synced document (names, relationship keys).
pub const MAX_NAME_BYTES: usize = 1024;

/// Largest vertex/index stream a single mesh write may upload.
pub const MAX_MESH_ELEMENTS: usize = 4 * 1024 * 1024;

/// Hard ceiling on live prims in one document's tree.
///
/// Unlike the rate-limited `Flow::CreatePrim`, this bounds the standing size of
/// the synced scene, and being read from the tree itself it falls as prims are
/// deleted.
pub const MAX_PRIMS_PER_DOC: usize = 50_000;

impl Limits {
    /// Caps for a single script's document: tight enough that one document
    /// cannot exhaust the client on its own.
    #[must_use]
    pub fn document() -> Self {
        let stock = HashMap::from([
            (Stock::WasmBytes, 256 * MIB),
            (Stock::Documents, 32),
            (Stock::Prims, 50_000),
            (Stock::Slots, 100_000),
            (Stock::PortalWatches, 64),
            (Stock::Receptors, 256),
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
                Flow::PortalOpen,
                FlowLimit {
                    capacity:       16.0,
                    refill_per_sec: 2.0,
                },
            ),
            (
                Flow::Emit,
                FlowLimit {
                    capacity:       256.0,
                    refill_per_sec: 128.0,
                },
            ),
            (
                Flow::Publish,
                FlowLimit {
                    capacity:       4.0,
                    refill_per_sec: 0.2,
                },
            ),
            (
                Flow::BlobUpload,
                FlowLimit {
                    capacity:       32.0,
                    refill_per_sec: 4.0,
                },
            ),
        ]);
        Self { stock, flow }
    }

    /// Caps for a whole space: the sum of every document placed in it, bounding
    /// the blast radius a single space can inflict on the client.
    #[must_use]
    pub fn space() -> Self {
        let stock = HashMap::from([
            (Stock::WasmBytes, 1024 * MIB),
            (Stock::Documents, 256),
            (Stock::Prims, 500_000),
            (Stock::Slots, 1_000_000),
            (Stock::PortalWatches, 512),
            (Stock::Receptors, 4_096),
        ]);
        let flow = HashMap::from([
            (
                Flow::CreateDocument,
                FlowLimit {
                    capacity:       32.0,
                    refill_per_sec: 4.0,
                },
            ),
            (
                Flow::CreatePrim,
                FlowLimit {
                    capacity:       8_000.0,
                    refill_per_sec: 2_000.0,
                },
            ),
            (
                Flow::PortalOpen,
                FlowLimit {
                    capacity:       64.0,
                    refill_per_sec: 8.0,
                },
            ),
            (
                Flow::Emit,
                FlowLimit {
                    capacity:       1_024.0,
                    refill_per_sec: 512.0,
                },
            ),
            (
                Flow::Publish,
                FlowLimit {
                    capacity:       16.0,
                    refill_per_sec: 1.0,
                },
            ),
            (
                Flow::BlobUpload,
                FlowLimit {
                    capacity:       128.0,
                    refill_per_sec: 16.0,
                },
            ),
        ]);
        Self { stock, flow }
    }

    /// Caps for everything attributable to one user across spaces. The
    /// outermost ring: a runaway user cannot escape it by fanning out into
    /// many spaces.
    #[must_use]
    pub fn user() -> Self {
        let stock = HashMap::from([
            (Stock::WasmBytes, 2048 * MIB),
            (Stock::Documents, 1_024),
            (Stock::Prims, 2_000_000),
            (Stock::Slots, 4_000_000),
            (Stock::PortalWatches, 2_048),
            (Stock::Receptors, 16_384),
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
                    capacity:       128.0,
                    refill_per_sec: 16.0,
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
                    capacity:       256.0,
                    refill_per_sec: 32.0,
                },
            ),
        ]);
        Self { stock, flow }
    }
}
