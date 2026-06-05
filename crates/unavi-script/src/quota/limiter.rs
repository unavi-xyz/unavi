use std::sync::Arc;

use wasmtime::ResourceLimiter;

use crate::quota::{
    Quota,
    Stock,
};

/// Binds a store's linear-memory growth to its document quota.
///
/// Growth past the [`Stock::WasmBytes`] cap is refused, which the guest sees as
/// an allocation failure. The charge is released in full when the store, and so
/// this limiter, is dropped.
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
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let want = desired as u64;
        if want <= self.charged {
            return Ok(true);
        }
        let delta = want - self.charged;
        match self.quota.try_charge(Stock::WasmMemory, delta) {
            Ok(()) => {
                self.charged = want;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    fn table_growing(
        &mut self,
        _current: usize,
        _desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(true)
    }
}

impl Drop for QuotaLimiter {
    fn drop(&mut self) {
        self.quota.release(Stock::WasmMemory, self.charged);
    }
}
