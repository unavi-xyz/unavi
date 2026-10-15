//! The mark this tool announces, drawn by the shell as a grid of dots.

use wired_guest::glyph::pack;

use crate::unavi::tool::api::Glyph;

/// A pistol in profile: barrel on top, grip hanging below.
///
/// Authored as rows so it can be read and edited as the picture it is; `#` is
/// a lit dot and anything else an unlit one.
const MARK: [&str; 9] = [
    ".........",
    ".........",
    ".#######.",
    ".########",
    ".###.....",
    ".##......",
    ".##......",
    ".........",
    ".........",
];

/// The announced mark as the record `unavi:tool/api` takes.
pub fn mark() -> Glyph {
    let (size, bits) = pack(&MARK);
    Glyph { size, bits }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mark_packs_to_its_exact_bytes() {
        let glyph = mark();
        assert_eq!(glyph.size, 9);
        assert_eq!(
            glyph.bits,
            [
                0x00, 0x00, 0xF8, 0xF3, 0xEF, 0xC0, 0x80, 0x01, 0x00, 0x00, 0x00
            ]
        );
    }
}
