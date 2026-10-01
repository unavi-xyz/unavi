//! Bounds a wasmtime store's memories and tables.

use std::sync::Arc;

use unavi_policy::quota::{
    Quota,
    Stock,
};
use wasmtime::ResourceLimiter;

/// Most elements one table may hold.
const MAX_TABLE_ELEMENTS: usize = 100_000;
/// Most core instances, memories and tables one store may create. A component
/// has one of each per core module, so these bound how far a guest can
/// multiply its allowance by instantiating more modules.
const MAX_INSTANCES: usize = 64;
const MAX_MEMORIES: usize = 16;
const MAX_TABLES: usize = 64;

/// Charges every linear memory's growth to the document's
/// [`Stock::WasmMemory`]. Growth past the cap is refused, which the guest sees
/// as an allocation failure. The charge is released when the store drops.
pub struct QuotaLimiter {
    quota:   Arc<Quota>,
    charged: u64,
}

impl QuotaLimiter {
    #[must_use]
    pub const fn new(quota: Arc<Quota>) -> Self {
        Self { quota, charged: 0 }
    }
}

impl ResourceLimiter for QuotaLimiter {
    /// Charges the growth of this one memory, so a component with several
    /// memories pays for each of them.
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let delta = desired.saturating_sub(current) as u64;
        if self.quota.charge(Stock::WasmMemory, delta).is_err() {
            return Ok(false);
        }
        self.charged = self.charged.saturating_add(delta);
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(desired <= MAX_TABLE_ELEMENTS)
    }

    fn instances(&self) -> usize {
        MAX_INSTANCES
    }

    fn memories(&self) -> usize {
        MAX_MEMORIES
    }

    fn tables(&self) -> usize {
        MAX_TABLES
    }
}

impl Drop for QuotaLimiter {
    fn drop(&mut self) {
        self.quota.release(Stock::WasmMemory, self.charged);
    }
}

#[cfg(test)]
mod tests {
    use unavi_policy::quota::limits::Limits;

    use super::*;

    fn quota(memory: u64) -> Arc<Quota> {
        let mut limits = Limits::default();
        limits.stock.insert(Stock::WasmMemory, memory);
        Quota::new(limits, None)
    }

    #[test]
    fn each_memory_is_charged_for_its_own_growth() {
        let quota = quota(100);
        let mut limiter = QuotaLimiter::new(Arc::clone(&quota));
        assert!(limiter.memory_growing(0, 40, None).expect("grow"));
        assert!(limiter.memory_growing(0, 40, None).expect("grow"));
        assert_eq!(quota.usage(Stock::WasmMemory), 80);
        assert!(
            !limiter.memory_growing(0, 40, None).expect("grow"),
            "a third memory of the same size is over the cap"
        );
        drop(limiter);
        assert_eq!(quota.usage(Stock::WasmMemory), 0);
    }

    #[test]
    fn a_table_cannot_grow_without_bound() {
        let mut limiter = QuotaLimiter::new(quota(0));
        assert!(
            !limiter
                .table_growing(0, MAX_TABLE_ELEMENTS + 1, None)
                .expect("grow")
        );
    }
}
