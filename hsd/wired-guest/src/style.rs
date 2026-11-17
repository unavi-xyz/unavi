//! The values every first-party surface draws with: colours in linear RGB,
//! type sizes in milliradians, cell dimensions, durations in seconds.
//!
//! Colours are written linear; the sRGB they come from is the launcher's
//! `tokens.css` and the app's own `style` module, which hold the same values.

use crate::math::Color;

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
pub const MRAD_SMALL: f32 = 14.0;
pub const MRAD_BODY: f32 = 20.0;
pub const MRAD_TITLE: f32 = 28.0;
pub const MRAD_DISPLAY: f32 = 40.0;
pub const MONO_ADVANCE: f32 = 0.6;
pub const MONO_LINE: f32 = 1.5;
pub const MONO_PAD_COLUMNS: f32 = 1.0;
pub const MONO_PAD_LINES: f32 = 0.5;
pub const MONO_COLUMNS: [u32; 4] = [24, 40, 64, 80];
pub const CELLS_FILL: f32 = 0.86;
pub const CELLS_LEVEL_OFF: f32 = 0.06;
pub const CELLS_LEVEL_EMBER: f32 = 0.45;
pub const CELLS_LEVEL_LIT: f32 = 1.0;
pub const CELLS_COLOUR: Color = FG;
pub const CELLS_ATTENDED: Color = BLUE;
pub const CELLS_TRAVEL: f32 = 90.0;
pub const CELLS_FRONT_FLASH: f32 = 0.85;
pub const CELLS_FRONT: f32 = 0.2;
pub const CELLS_WAKE_BOOST: f32 = 0.2;
pub const CELLS_WAKE_RADIUS: u32 = 7;
pub const CELLS_BREATH: f32 = 0.02;
pub const CELLS_BREATH_RATE: f32 = 1.2;
pub const CELLS_SOFT: f32 = 0.04;
pub const CELLS_MOIRE_PX: f32 = 2.0;
pub const CELLS_GLYPH_WIDTH: u32 = 12;
pub const CELLS_KEY_WIDTH: u32 = 16;
pub const CELLS_BED_WIDTH: u32 = 58;
pub const CELLS_BED_GAP: u32 = 3;
pub const CELLS_BED_MARGIN: u32 = 2;
pub const CELLS_CELL_ROW: u32 = 12;
pub const CELLS_VOID_FLOOR: f32 = 0.45;
pub const FROST_GROUND: Color = Color::rgb(1.0, 1.0, 1.0);
pub const FROST_GROUND_ALPHA: f32 = 0.64;
pub const FROST_BLUR_PX: f32 = 18.0;
pub const FROST_BLUR_UV: f32 = 0.006;
pub const FROST_EDGE_WIDTH: f32 = 0.006;
pub const FROST_INK: Color = FG;
pub const FROST_INK_2: Color = Color::rgb(0.102_241_73, 0.102_241_73, 0.132_868_32);
pub const FROST_HAIRLINE: Color = FG;
pub const FROST_HAIRLINE_ALPHA: f32 = 0.13;
pub const FROST_FILL: Color = FG;
pub const FROST_FILL_ALPHA: f32 = 0.04;
pub const FROST_EDGE: Color = Color::rgb(1.0, 1.0, 1.0);
pub const FROST_EDGE_ALPHA: f32 = 0.95;
pub const FROST_RADIUS: f32 = 0.08;
pub const FROST_SHADOW: Color = Color::rgb(0.122_138_776, 0.097_587_35, 0.351_532_6);
pub const FROST_SHADOW_ALPHA: f32 = 0.3;
pub const FROST_SHADOW_DX_PX: f32 = 12.0;
pub const FROST_SHADOW_DY_PX: f32 = 16.0;
pub const MOTION_FLIP: f32 = 0.3;
pub const MOTION_ARRIVE: f32 = 0.7;
