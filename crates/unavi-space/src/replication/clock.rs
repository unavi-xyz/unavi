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

/// Whether `at` runs no more than [`MAX_SKEW_MICROS`] past `recv`.
#[must_use]
pub const fn within_skew(at: u64, recv: u64) -> bool {
    at <= recv.saturating_add(MAX_SKEW_MICROS)
}

/// The stamp for a local statement that has to outrank `newest`: now, or just
/// past `newest`, but never so far ahead that a receiver within half the skew
/// allowance refuses it.
#[must_use]
pub const fn stamp_after(newest: Option<u64>, now: u64) -> u64 {
    let lead = now.saturating_add(MAX_SKEW_MICROS / 2);
    let next = match newest {
        Some(newest) => newest.saturating_add(1),
        None => 0,
    };
    let at = if next > now { next } else { now };
    if at > lead { lead } else { at }
}

/// `at` clamped to no later than [`MAX_SKEW_MICROS`] past `recv`.
#[must_use]
pub const fn clamp_to(at: u64, recv: u64) -> u64 {
    let limit = recv.saturating_add(MAX_SKEW_MICROS);
    if at > limit { limit } else { at }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_local_stamp_outranks_the_newest_without_outrunning_half_the_skew() {
        let now = 1_000_000_000;
        assert_eq!(stamp_after(None, now), now);
        assert_eq!(stamp_after(Some(now - 5), now), now);
        assert_eq!(stamp_after(Some(now + 5), now), now + 6);
        assert_eq!(
            stamp_after(Some(u64::MAX), now),
            now + MAX_SKEW_MICROS / 2,
            "a forged far-future hold is not chased past what receivers accept"
        );
    }
}
