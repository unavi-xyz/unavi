//! `wired:core/time`: the local clock's relation to UTC.

/// Minutes east of UTC right now, daylight saving included.
#[cfg(not(target_family = "wasm"))]
#[must_use]
pub fn utc_offset() -> i32 {
    chrono::Local::now().offset().local_minus_utc() / 60
}

/// Minutes east of UTC right now, daylight saving included.
///
/// `getTimezoneOffset` counts minutes west, so it is negated.
#[cfg(target_family = "wasm")]
#[must_use]
pub fn utc_offset() -> i32 {
    -(js_sys::Date::new_0().get_timezone_offset().round() as i32)
}
