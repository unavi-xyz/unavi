//! The time, drawn on the root's face.
//!
//! A guest has the wall clock but no timezone data, so this is UTC until the
//! host can say what the local offset is.

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

/// The minute last shown, so the readout is only rewritten — and only flips —
/// when the minute turns over.
#[derive(Default)]
pub struct Clock {
    shown: Option<u64>,
}

impl Clock {
    /// The readout's new text, or `None` while the minute it shows is still
    /// the current one.
    pub fn tick(&mut self) -> Option<String> {
        let minute = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() / 60;
        if self.shown == Some(minute) {
            return None;
        }
        self.shown = Some(minute);
        Some(face(minute))
    }
}

/// Hour over minute, twenty-four hour: a readout line holds two characters
/// when there are two lines, and two lines of two read faster than one of
/// five.
fn face(minute: u64) -> String {
    let of_day = minute % (24 * 60);
    format!("{:02}\n{:02}", of_day / 60, of_day % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midnight_reads_as_zeros() {
        assert_eq!(face(0), "00\n00");
    }

    #[test]
    fn the_hour_sits_over_the_minute() {
        assert_eq!(face(10 * 60 + 42), "10\n42");
    }

    #[test]
    fn days_wrap_back_to_midnight() {
        assert_eq!(face(3 * 24 * 60 + 23 * 60 + 59), "23\n59");
        assert_eq!(face(4 * 24 * 60), "00\n00");
    }
}
