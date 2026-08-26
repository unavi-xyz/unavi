use std::time::Duration;

use bevy::ecs::component::Component;
use unavi_quota::{
    Flow,
    Quota,
    QuotaError,
    Reservation,
    StockGuard,
};

#[cfg(not(target_family = "wasm"))] pub mod limiter;

/// How long a script may be slowed down before the ask is called unreasonable.
///
/// One ceiling rather than one per lifecycle phase; nothing carries the phase
/// to this boundary yet.
pub const MAX_FLOW_WAIT: Duration = Duration::from_secs(10);

/// Longest single sleep, so a script that is cancelled mid-wait notices
/// reasonably soon.
const MAX_SLEEP: Duration = Duration::from_millis(250);

/// Spends `n` of `flow`, waiting for the bucket to refill rather than failing.
///
/// Sound only for [`Flow`] — a [`unavi_quota::Stock`] is released by something
/// else acting, so waiting on one is a deadlock, not backpressure, and those
/// keep erroring.
///
/// Nothing is taken until the reservation says `Ready`, so dropping this
/// future leaves every bucket untouched.
pub async fn acquire(quota: &Quota, flow: Flow, n: f64) -> Result<(), QuotaError> {
    let mut waited = Duration::ZERO;
    loop {
        match quota.reserve(flow, n) {
            Reservation::Ready => {
                quota.commit(flow, n);
                return Ok(());
            }
            // Fails fast rather than after the ceiling elapses: an ask larger
            // than a bucket's whole capacity is unsatisfiable at any time.
            Reservation::Never => return Err(QuotaError::Flow(flow)),
            Reservation::After(wait) => {
                if waited.saturating_add(wait) > MAX_FLOW_WAIT {
                    return Err(QuotaError::Flow(flow));
                }
                let sleep = wait.min(MAX_SLEEP);
                waited = waited.saturating_add(sleep);
                n0_future::time::sleep(sleep).await;
            }
        }
    }
}

#[derive(Component, Default)]
pub struct QuotaGuards(pub Vec<StockGuard>);

/// Marks a document whose scripts bypass quota enforcement, for trusted system
/// scripts.
#[derive(Component)]
pub struct QuotaExempt;
