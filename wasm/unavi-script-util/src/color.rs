use blake3::Hash;
use wired_prelude::prelude::Color;

#[must_use]
pub fn generate_color(hash: Hash) -> Color {
    color_from_hash(hash, 0.70, 0.25, 0.70, 0.25)
}

#[must_use]
pub fn generate_muted_color(hash: Hash) -> Color {
    color_from_hash(hash, 0.25, 0.20, 0.55, 0.25)
}

fn color_from_hash(hash: Hash, s_base: f32, s_range: f32, v_base: f32, v_range: f32) -> Color {
    let bytes = hash.as_slice();

    let hue_u64 = u64::from_le_bytes(bytes[0..8].try_into().expect("u64"));
    let h = (hue_u64 as f64 / u64::MAX as f64) as f32;

    let s_u16 = u16::from_le_bytes(bytes[8..10].try_into().expect("u16"));
    let v_u16 = u16::from_le_bytes(bytes[10..12].try_into().expect("u16"));

    let s = (f32::from(s_u16) / f32::from(u16::MAX)).mul_add(s_range, s_base);
    let v = (f32::from(v_u16) / f32::from(u16::MAX)).mul_add(v_range, v_base);

    Color::hsv(h, s, v)
}
