//! The values the app draws with outside a script: colours in linear RGB,
//! durations in seconds. The guest library holds the same values in
//! `wired_guest::style`.

use bevy::color::Color as BevyColor;

/// A colour in linear RGB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[must_use]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

impl From<Color> for BevyColor {
    fn from(color: Color) -> Self {
        Self::linear_rgba(color.r, color.g, color.b, color.a)
    }
}

pub const BG: Color = Color::rgb(0.938_685_7, 0.938_685_7, 0.913_098_63);
pub const BG_DARK: Color = Color::rgb(0.003_035_27, 0.003_035_27, 0.003_035_27);
pub const FG: Color = Color::rgb(0.004_776_953_3, 0.004_776_953_3, 0.004_776_953_3);
pub const FG_2: Color = Color::rgb(0.042_311_41, 0.042_311_41, 0.042_311_41);
pub const MUTED: Color = Color::rgb(0.132_868_32, 0.132_868_32, 0.132_868_32);
pub const LIGHT: Color = Color::rgb(0.806_952_24, 0.806_952_24, 0.768_151_16);
pub const PANEL: Color = Color::rgb(1.0, 1.0, 1.0);
pub const TINT: Color = Color::rgb(0.854_992_6, 0.854_992_6, 0.806_952_24);
pub const RULE: Color = Color::rgb(0.715_693_5, 0.715_693_5, 0.665_387_3);
pub const BLUE: Color = Color::rgb(0.018_500_22, 0.124_771_82, 0.830_769_9);
pub const RED: Color = Color::rgb(0.715_693_5, 0.019_382_361, 0.019_382_361);
pub const DUR_PRESS: f32 = 0.1;
pub const DUR_HOVER: f32 = 0.18;
pub const FIELD_COVER: Color = Color::rgb(0.775_822_2, 0.644_479_7, 0.737_910_4);
pub const FIELD_SHADE_HUE: f32 = 290.0;
pub const FIELD_SHADE_CHROMA: f32 = 0.04;
pub const FIELD_SKY_CHROMA_MAX: f32 = 0.05;
pub const MOTION_FLIP: f32 = 0.3;
pub const MOTION_ARRIVE: f32 = 0.7;
