//! Colour helpers shared across guests.

use blake3::Hash;

use crate::math::Color;

/// A `document-id`'s four words packed little-endian into the 32 bytes of the
/// namespace, which is what [`generate_color`] takes and what a registry hex
/// encodes.
#[must_use]
pub fn document_seed(id: (u64, u64, u64, u64)) -> Hash {
    let mut bytes = [0u8; 32];
    bytes[0..8].copy_from_slice(&id.0.to_le_bytes());
    bytes[8..16].copy_from_slice(&id.1.to_le_bytes());
    bytes[16..24].copy_from_slice(&id.2.to_le_bytes());
    bytes[24..32].copy_from_slice(&id.3.to_le_bytes());
    Hash::from_bytes(bytes)
}

/// Derives a stable, pleasant colour from a hash, such as a document id.
#[must_use]
pub fn generate_color(hash: Hash) -> Color {
    let bytes = hash.as_slice();

    let hue_u64 = u64::from_le_bytes(bytes[0..8].try_into().unwrap_or_default());
    let h = (hue_u64 as f64 / u64::MAX as f64) as f32;

    let s_u16 = u16::from_le_bytes(bytes[8..10].try_into().unwrap_or_default());
    let v_u16 = u16::from_le_bytes(bytes[10..12].try_into().unwrap_or_default());

    let s = (f32::from(s_u16) / f32::from(u16::MAX)).mul_add(0.25, 0.70);
    let v = (f32::from(v_u16) / f32::from(u16::MAX)).mul_add(0.25, 0.70);

    Color::hsv(h, s, v)
}

/// A document's colour, the one its gate and beacon wear, which marks it
/// wherever it is named.
#[must_use]
pub fn document_color(id: (u64, u64, u64, u64)) -> Color {
    generate_color(document_seed(id))
}

/// A user's colour, derived from their DID so every peer sees the same one.
#[must_use]
pub fn user_color(did: &str) -> Color {
    generate_color(blake3::hash(did.as_bytes()))
}

/// A document's first three bytes in hex, the short id the devtools print.
#[must_use]
pub fn short_id(id: (u64, u64, u64, u64)) -> String {
    let [a, b, c, ..] = id.0.to_le_bytes();
    format!("{a:02x}{b:02x}{c:02x}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_users_colour_follows_their_did() {
        assert_eq!(user_color("did:key:a"), user_color("did:key:a"));
        assert_ne!(user_color("did:key:a"), user_color("did:key:b"));
    }

    #[test]
    fn a_short_id_is_the_leading_bytes_of_the_namespace() {
        let id = (
            u64::from_le_bytes([0xAB, 0x01, 0xFF, 9, 9, 9, 9, 9]),
            1,
            2,
            3,
        );
        assert_eq!(short_id(id), "ab01ff");
        assert_eq!(&document_seed(id).to_hex()[..6], "ab01ff");
    }
}
