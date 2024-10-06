use std::sync::atomic::{AtomicU32, Ordering};

static ID: AtomicU32 = AtomicU32::new(0);

/// Globally unique ID for a resource, constructed using `default`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ResourceId(u32);

impl Default for ResourceId {
    fn default() -> Self {
        Self(ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl From<ResourceId> for u32 {
    fn from(val: ResourceId) -> Self {
        val.0
    }
}
