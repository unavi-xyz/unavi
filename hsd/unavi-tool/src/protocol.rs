//! Wire payloads and channel names shared by the tool and shell sides of
//! `unavi:tool/api`. The WIT is the contract; this is its wire format.

use serde::{
    Deserialize,
    Serialize,
};
use wired_guest::math::Color;

pub const CH_DISCOVER: &str = "unavi:tool/discover";
pub const CH_REGISTER: &str = "unavi:tool/register";
pub const CH_ACTIVATE: &str = "unavi:tool/activate";
pub const CH_DEACTIVATE: &str = "unavi:tool/deactivate";
pub const CH_SET_STATE: &str = "unavi:tool/set-state";
pub const CH_TRIGGER: &str = "unavi:tool/trigger";
pub const CH_SCROLL: &str = "unavi:tool/scroll";

/// Longest `tool` name, matching the bound documented on
/// `unavi:tool/api.tool.constructor`.
pub const MAX_NAME_BYTES: usize = 48;
/// Longest `tool` description, matching the bound documented on
/// `unavi:tool/api.tool.constructor`.
pub const MAX_DESCRIPTION_BYTES: usize = 256;

/// Largest `glyph` side, matching the bound documented on
/// `unavi:tool/api.glyph`.
pub const MAX_GLYPH_SIZE: u8 = 13;

#[derive(Serialize, Deserialize)]
pub struct RegisterPayload {
    pub name:        String,
    pub description: String,
    pub glyph:       Option<GlyphPayload>,
}

#[derive(Serialize, Deserialize)]
pub struct GlyphPayload {
    pub size: u8,
    pub bits: Vec<u8>,
}

/// Bytes a glyph of side `size` occupies, one bit per cell.
const fn glyph_bytes(size: u8) -> usize {
    (size as usize).pow(2).div_ceil(8)
}

/// What the registry makes of a glyph that arrived in a `register` payload: it
/// is kept, cut to the bytes its grid uses, when it passes [`check_glyph`] and
/// dropped otherwise. Truncating bounds what a hand-crafted emitter can make
/// the shell hold, as name and description are bounded.
pub fn accept_glyph(glyph: GlyphPayload) -> Option<GlyphPayload> {
    check_glyph(glyph.size, &glyph.bits).ok()?;
    let mut bits = glyph.bits;
    bits.truncate(glyph_bytes(glyph.size));
    Some(GlyphPayload {
        size: glyph.size,
        bits,
    })
}

/// Whether `size` and `bits` describe a glyph the WIT allows. The tool
/// constructor refuses one that does not, and the registry applies the same
/// rule to what arrives, since a `register` payload may come from an emitter
/// that never went through the constructor.
pub fn check_glyph(size: u8, bits: &[u8]) -> Result<(), String> {
    if !(1..=MAX_GLYPH_SIZE).contains(&size) {
        return Err(format!("glyph size must be 1 to {MAX_GLYPH_SIZE}"));
    }
    let needed = glyph_bytes(size);
    if bits.len() < needed {
        return Err(format!(
            "a glyph of size {size} needs at least {needed} bytes of bits"
        ));
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct ToolStatePayload {
    pub color: Color,
}

#[derive(Serialize, Deserialize)]
pub struct TriggerPayload {
    pub pressed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ScrollPayload {
    pub delta: f32,
}

/// Shortens `s` to at most `max_bytes`, at a char boundary. A `register`
/// payload comes from another document; the `tool` constructor bounds its
/// own name and description, but nothing stops a hand-crafted emitter from
/// sending more.
pub fn truncate(s: &mut String, max_bytes: usize) {
    if s.len() <= max_bytes {
        return;
    }
    let mut end = max_bytes;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s.truncate(end);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_glyph_of_the_right_size_with_enough_bits_passes() {
        assert!(check_glyph(9, &[0; 11]).is_ok());
        assert!(check_glyph(13, &[0; 22]).is_ok());
        assert!(check_glyph(1, &[0]).is_ok());
    }

    #[test]
    fn bytes_past_what_the_grid_needs_are_ignored() {
        assert!(check_glyph(9, &[0; 40]).is_ok());
    }

    #[test]
    fn an_accepted_glyph_is_cut_to_the_bytes_its_grid_uses() {
        let glyph = accept_glyph(GlyphPayload {
            size: 9,
            bits: vec![0xFF; 40],
        })
        .expect("valid");
        assert_eq!(glyph.bits.len(), 11);
    }

    #[test]
    fn a_glyph_that_breaks_the_rules_is_not_accepted() {
        assert!(
            accept_glyph(GlyphPayload {
                size: 14,
                bits: vec![0; 40],
            })
            .is_none()
        );
        assert!(
            accept_glyph(GlyphPayload {
                size: 9,
                bits: vec![0; 10],
            })
            .is_none()
        );
    }

    #[test]
    fn a_size_outside_one_to_thirteen_is_refused() {
        assert!(check_glyph(0, &[0; 32]).is_err());
        assert!(check_glyph(14, &[0; 32]).is_err());
    }

    /// 9 x 9 is 81 cells, which is 11 bytes and not 10.
    #[test]
    fn too_few_bytes_for_the_grid_is_refused() {
        assert!(check_glyph(9, &[0; 10]).is_err());
        assert!(check_glyph(13, &[0; 21]).is_err());
    }
}
