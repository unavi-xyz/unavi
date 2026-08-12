use blake3::Hash;
use wired_prelude::prelude::Color;

#[must_use]
pub fn generate_color(hash: Hash) -> Color {
    let bytes = hash.as_slice();

    let hue_u64 = u64::from_le_bytes(bytes[0..8].try_into().expect("u64"));
    let h = (hue_u64 as f64 / u64::MAX as f64) as f32;

    let s_u16 = u16::from_le_bytes(bytes[8..10].try_into().expect("u16"));
    let v_u16 = u16::from_le_bytes(bytes[10..12].try_into().expect("u16"));

    let s = (f32::from(s_u16) / f32::from(u16::MAX)).mul_add(0.25, 0.70);
    let v = (f32::from(v_u16) / f32::from(u16::MAX)).mul_add(0.25, 0.70);

    Color::hsv(h, s, v)
}

/// Mixes `color` toward its own luminance gray.
#[must_use]
pub fn desaturate(color: Color, amount: f32) -> Color {
    let luma = 0.114f32.mul_add(color.b, 0.299f32.mul_add(color.r, 0.587 * color.g));
    Color {
        r: color.r.mul_add(1.0 - amount, luma * amount),
        g: color.g.mul_add(1.0 - amount, luma * amount),
        b: color.b.mul_add(1.0 - amount, luma * amount),
        a: color.a,
    }
}
