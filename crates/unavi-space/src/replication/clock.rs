//! Wall-clock stamps for replicated state.

use web_time::{
    SystemTime,
    UNIX_EPOCH,
};

/// How far ahead of local time a peer's session stamp may run. Tight, since a
/// future stamp wins every last-write-wins merge until real time catches up.
pub const MAX_SKEW_MICROS: u64 = 2 * 1_000_000;

#[must_use]
pub fn current_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_micros()).unwrap_or(u64::MAX))
}

/// `at` clamped to no later than [`MAX_SKEW_MICROS`] past `recv`.
#[must_use]
pub const fn clamp_to(at: u64, recv: u64) -> u64 {
    let limit = recv.saturating_add(MAX_SKEW_MICROS);
    if at > limit { limit } else { at }
}
