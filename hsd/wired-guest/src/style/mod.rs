//! Design tokens as constants: colours in linear RGB, type sizes in
//! milliradians, cell dimensions, durations in seconds.
//!
//! Generated at build time from `assets/tokens.css` and `assets/tokens.toml`.
//! Names are the CSS property upper-cased (`BG`, `FG_2`, `DUR_PRESS`) or the
//! `tokens.toml` section and key (`MRAD_BODY`, `CELLS_FILL`).

use crate::math::Color;

include!(concat!(env!("OUT_DIR"), "/tokens.rs"));

#[cfg(test)]
#[path = "../../../../assets/tokens_gen.rs"]
mod generator;

#[cfg(test)]
mod tests {
    use super::{
        generator::{
            generate,
            srgb_to_linear,
        },
        *,
    };

    #[test]
    fn the_shipped_token_files_generate() {
        let css = include_str!("../../../../assets/tokens.css");
        let toml = include_str!("../../../../assets/tokens.toml");
        assert_eq!(generate(css, toml).map(|out| out.is_empty()), Ok(false));
    }

    #[test]
    fn the_ground_is_the_websites_ground_in_linear() {
        let expected = srgb_to_linear(f64::from(0xF8_u8) / 255.0) as f32;
        assert!((BG.r - expected).abs() < 1.0e-6);
        assert!((BG.b - srgb_to_linear(f64::from(0xF5_u8) / 255.0) as f32).abs() < 1.0e-6);
    }

    #[test]
    fn the_type_scale_ascends() {
        const {
            assert!(MRAD_SMALL < MRAD_BODY && MRAD_BODY < MRAD_TITLE && MRAD_TITLE < MRAD_DISPLAY);
        }
    }
}
