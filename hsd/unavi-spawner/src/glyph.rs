//! The mark this tool announces, drawn by the shell as a grid of dots.

use wired_guest::glyph::pack;

use crate::unavi::tool::api::Glyph;

/// A plus: something added.
///
/// Authored as rows so it can be read and edited as the picture it is; `#` is
/// a lit dot and anything else an unlit one.
const MARK: [&str; 9] = [
    ".........",
    "....#....",
    "....#....",
    "....#....",
    ".#######.",
    "....#....",
    "....#....",
    "....#....",
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
                0x00, 0x20, 0x40, 0x80, 0xE0, 0x0F, 0x02, 0x04, 0x08, 0x00, 0x00
            ]
        );
    }
}
