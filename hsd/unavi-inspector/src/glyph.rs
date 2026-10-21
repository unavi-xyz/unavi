//! The mark this tool announces, drawn by the shell as a grid of dots.

use wired_guest::glyph::pack;

use crate::unavi::tool::api::Glyph;

/// A magnifying glass: a ring and its handle.
///
/// Authored as rows so it can be read and edited as the picture it is; `#` is
/// a lit dot and anything else an unlit one.
const MARK: [&str; 9] = [
    ".........",
    "..####...",
    ".#....#..",
    ".#....#..",
    ".#....#..",
    "..####...",
    "......#..",
    ".......#.",
    ".........",
];

/// The announced mark as the record `unavi:tool/api` takes.
pub fn mark() -> Glyph {
    let (size, bits) = pack(&MARK);
    Glyph { size, bits }
}
