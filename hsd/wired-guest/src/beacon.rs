//! The beacon-to-gate contract: a beacon emits its space's id on
//! [`CHANNEL`]; a gate nearby listens on the same channel and opens to it.

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

/// Global event channel a beacon emits its 32-byte space id on.
pub const CHANNEL: &str = "unavi:beacon/id";

/// The custom property a minted beacon commits to say when it was planted.
///
/// Named `group/field`. Every peer reads the same stamp, so every peer agrees
/// how far into its hover a beacon is, however late it joined.
pub const PLANTED_KEY: &str = "unavi/planted";

/// The wall clock in milliseconds since the Unix epoch, which is what
/// [`planted_bytes`] stamps.
#[must_use]
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}

/// `unix_ms` as the bytes committed under [`PLANTED_KEY`].
#[must_use]
pub const fn planted_bytes(unix_ms: u64) -> [u8; 8] {
    unix_ms.to_le_bytes()
}

/// Seconds between a committed [`PLANTED_KEY`] stamp and `now_ms`. A stamp
/// from the future, which a peer with a slow clock reads, counts as just
/// planted. None when the bytes are not a stamp.
#[must_use]
pub fn age_secs(planted: &[u8], now_ms: u64) -> Option<f32> {
    let planted = u64::from_le_bytes(planted.try_into().ok()?);
    Some(now_ms.saturating_sub(planted) as f32 / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stamp_reads_back_as_its_age() {
        let stamp = planted_bytes(10_000);
        assert_eq!(age_secs(&stamp, 12_500), Some(2.5));
    }

    #[test]
    fn a_future_stamp_is_just_planted() {
        let stamp = planted_bytes(10_000);
        assert_eq!(age_secs(&stamp, 9_000), Some(0.0));
    }

    #[test]
    fn bytes_that_are_not_a_stamp_have_no_age() {
        assert_eq!(age_secs(&[1, 2, 3], 5), None);
    }
}
