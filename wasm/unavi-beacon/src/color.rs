use blake3::Hash;
use wired_prelude::prelude::*;

pub fn generate_beacon_color(hash: Hash) -> Color {
    let bytes = hash.as_slice();

    // Use first 8 bytes for hue (full spectrum)
    let hue_u64 = u64::from_le_bytes(bytes[0..8].try_into().expect("u64"));
    let h = (hue_u64 as f64 / u64::MAX as f64) as f32; // 0..1

    // Next bytes for saturation/value but clamp to nice ranges
    let s_u16 = u16::from_le_bytes(bytes[8..10].try_into().expect("u16"));
    let v_u16 = u16::from_le_bytes(bytes[10..12].try_into().expect("u16"));

    // Keep colors vivid but not neon / washed out
    let s = (f32::from(s_u16) / f32::from(u16::MAX)).mul_add(0.35, 0.55);
    let v = (f32::from(v_u16) / f32::from(u16::MAX)).mul_add(0.30, 0.65);

    Color::hsv(h, s, v)
}
