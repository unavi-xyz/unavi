//! The local time, as the root page reads it.
//!
//! A guest has the wall clock but no timezone data, so the host says how far
//! the local clock is from UTC.

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use crate::wired::core::time::utc_offset;

const MINUTES_PER_DAY: i64 = 24 * 60;

/// The local time as `HH:MM`, twenty-four hour, or `None` while the host's
/// clock reads before the epoch.
///
/// The offset is read afresh each call, so a daylight saving change or a
/// traveller's new timezone shows within the minute it happens. A zone
/// shifts by whole minutes, so local minutes turn over exactly when UTC ones
/// do.
pub fn time() -> Option<String> {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let minute = i64::try_from(secs / 60).ok()?;
    Some(face(minute, utc_offset()))
}

/// `utc_minute` is minutes since the epoch; `offset` is minutes east of UTC.
fn face(utc_minute: i64, offset: i32) -> String {
    let of_day = (utc_minute + i64::from(offset)).rem_euclid(MINUTES_PER_DAY);
    format!("{:02}:{:02}", of_day / 60, of_day % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midnight_reads_as_zeros() {
        assert_eq!(face(0, 0), "00:00");
    }

    #[test]
    fn the_hour_sits_before_the_minute() {
        assert_eq!(face(10 * 60 + 42, 0), "10:42");
    }

    #[test]
    fn days_wrap_back_to_midnight() {
        assert_eq!(face(3 * MINUTES_PER_DAY + 23 * 60 + 59, 0), "23:59");
        assert_eq!(face(4 * MINUTES_PER_DAY, 0), "00:00");
    }

    #[test]
    fn an_east_offset_runs_ahead_of_utc() {
        assert_eq!(face(10 * 60, 60), "11:00");
        assert_eq!(face(10 * 60, 5 * 60 + 30), "15:30");
    }

    #[test]
    fn a_west_offset_runs_behind_utc() {
        assert_eq!(face(10 * 60, -5 * 60), "05:00");
        assert_eq!(face(10 * 60 + 15, -(3 * 60 + 30)), "06:45");
    }

    #[test]
    fn an_offset_carries_across_midnight_either_way() {
        assert_eq!(face(23 * 60 + 30, 60), "00:30");
        assert_eq!(face(MINUTES_PER_DAY + 30, -60), "23:30");
        assert_eq!(face(30, -60), "23:30");
    }
}
