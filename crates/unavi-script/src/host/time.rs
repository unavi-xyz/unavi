//! `wired:core/time`: the local clock's relation to UTC.

use jiff::Zoned;

/// Minutes east of UTC right now, daylight saving included.
///
/// The host's timezone comes from `TimeZone::system`: the platform database
/// on native, `Intl.DateTimeFormat` on the web.
#[must_use]
pub fn utc_offset() -> i32 {
    Zoned::now().offset().seconds() / 60
}
