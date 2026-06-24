use std::sync::Arc;

use unavi_quota::{
    Quota,
    Stock,
};
use wasmtime::ResourceLimiter;

/// Binds a store's linear-memory growth to its document quota. Growth past the
/// [`Stock::WasmMemory`] cap is refused (a guest allocation failure); the charge
/// releases when the store and this limiter drop.
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
