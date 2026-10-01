//! How scripts spend their document's quota.

use bevy::ecs::component::Component;
use unavi_policy::quota::{
    Flow,
    Quota,
    Reservation,
    StockLease,
};

use crate::error::ScriptError;

#[cfg(not(target_family = "wasm"))] pub mod limiter;

/// Spends `n` of `flow`, or fails at once when the bucket cannot give it now.
///
/// A host call never waits for a bucket to refill: a wait would stall the
/// script's whole tick, and the script is better placed to decide what to do
/// instead.
pub fn take(quota: &Quota, flow: Flow, n: u32) -> Result<(), ScriptError> {
    match quota.try_take(flow, n) {
        Reservation::Ready => Ok(()),
        Reservation::After(_) | Reservation::Never => Err(ScriptError::RateLimited(flow)),
    }
}

/// Stock a document holds for as long as it lives, such as the
/// [`unavi_policy::quota::Stock::Documents`] unit a script-minted document
/// costs.
#[derive(Component, Default)]
pub struct QuotaLeases {
    /// Released when the component drops.
    _held: Vec<StockLease>,
}

impl QuotaLeases {
    #[must_use]
    pub const fn new(leases: Vec<StockLease>) -> Self {
        Self { _held: leases }
    }
}

/// Marks a document whose scripts bypass quota enforcement, for trusted system
/// scripts.
#[derive(Component)]
pub struct QuotaExempt;
