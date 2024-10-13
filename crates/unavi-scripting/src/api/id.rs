use std::sync::atomic::{AtomicU32, Ordering};

static ID: AtomicU32 = AtomicU32::new(0);

/// Globally unique ID, constructed using `default`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct UniqueId(u32);

impl Default for UniqueId {
    fn default() -> Self {
        Self(ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl From<UniqueId> for u32 {
    fn from(val: UniqueId) -> Self {
        val.0
    }
}
