use web_time::{
    SystemTime,
    UNIX_EPOCH,
};

/// Caps peer-supplied timestamps to within the clock skew of local time, so a
/// forged future `at` cannot pin ownership/authority or win KV merges forever.
const MAX_CLOCK_SKEW_MILLIS: u64 = 5 * 60 * 1000;

#[must_use]
pub fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Whether `at` is within the accepted clock skew of local time.
#[must_use]
pub fn time_valid(at: u64) -> bool {
    // TODO lower bound check or use "recieved" time only
    at <= current_millis().saturating_add(MAX_CLOCK_SKEW_MILLIS)
}
