//! Exact text measurement for scripts, with no host round trip.
//!
//! | Module | Measures |
//! | --- | --- |
//! | `mono` | Monospace text in whole cells |
//! | `signage` | Montserrat 700 titles in ems |
//! | `scale` | A size step in milliradians as an em height in metres |

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "panes are the first consumer of measurement")
)]

pub mod mono;
mod montserrat;
pub mod scale;
pub mod signage;
