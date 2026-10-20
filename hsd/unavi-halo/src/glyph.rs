//! What each mote wears on its face: a grid of dots, authored as the picture
//! it is.
//!
//! VUI draws a glyph from packed bits, which nobody can read or edit by eye,
//! so the bitmaps live here as rows of text and are packed once on the way in.
//! They are all nine cells square and stay inside the circle VUI clips to,
//! because a lit dot out in a corner would be silently dropped.

use wired_guest::glyph::pack;

use crate::unavi::{
    tool::api as tool,
    vui::api::Glyph,
};

/// Rows of a glyph, top to bottom. `#` is a lit dot and anything else is not.
pub type Rows = [&'static str; 9];

pub const HOME: Rows = [
    ".........",
    "....#....",
    "...###...",
    "..#####..",
    ".#######.",
    "..##.##..",
    "..##.##..",
    "..##.##..",
    ".........",
];

pub const NAV: Rows = [
    ".........",
    "...###...",
    "..#...#..",
    ".#.....#.",
    ".#..#..#.",
    ".#.....#.",
    "..#...#..",
    "...###...",
    ".........",
];

pub const TOOLS: Rows = [
    ".........",
    ".#######.",
    ".#######.",
    "....#....",
    "....#....",
    "....#....",
    "....#....",
    "....#....",
    ".........",
];

/// You: a head and shoulders.
pub const SELF: Rows = [
    ".........",
    "...###...",
    "...###...",
    "...###...",
    ".........",
    "..#####..",
    ".#######.",
    ".#######.",
    ".........",
];

/// What you see through: a band with a clear pane in it.
pub const VISOR: Rows = [
    ".........",
    ".........",
    ".#######.",
    "#########",
    "#.......#",
    "#########",
    ".#######.",
    ".........",
    ".........",
];

/// The travel lens: a doorway with a point of light in it.
pub const LENS_TRAVEL: Rows = [
    ".........",
    "...###...",
    "..#...#..",
    "..#...#..",
    "..#.#.#..",
    "..#...#..",
    "..#...#..",
    "..#####..",
    ".........",
];

/// The social lens: a person.
pub const LENS_SOCIAL: Rows = [
    ".........",
    "...###...",
    "..#####..",
    "...###...",
    ".........",
    "..#####..",
    ".#######.",
    ".#######.",
    ".........",
];

/// The mark of the lens named `id`; a lens this halo has no mark for wears the
/// visor.
#[must_use]
pub fn lens(id: &str) -> &'static Rows {
    match id {
        unavi_lens::TRAVEL => &LENS_TRAVEL,
        unavi_lens::SOCIAL => &LENS_SOCIAL,
        _ => &VISOR,
    }
}

/// The space you are standing in: a map pin, worn in place of its beacon.
pub const HERE: Rows = [
    ".........",
    "...###...",
    "..#####..",
    "..##.##..",
    "..#####..",
    "...###...",
    "....#....",
    "....#....",
    ".........",
];

/// A space and the marker you can drop in it.
pub const BEACON: Rows = [
    ".........",
    ".##...##.",
    ".#.....#.",
    "....#....",
    "...###...",
    "....#....",
    ".#.....#.",
    ".##...##.",
    ".........",
];

/// Going to a space, or asking for the one you are in again: one arrow,
/// because both are the same act — leave and arrive.
pub const TRAVEL: Rows = [
    ".........",
    "....#....",
    "....##...",
    "....###..",
    ".#######.",
    "....###..",
    "....##...",
    "....#....",
    ".........",
];

/// What a tool wears when it announced no mark of its own.
pub const TOOL: Rows = [
    "....#....",
    "....#....",
    ".#..#..#.",
    "..#.#.#..",
    "#########",
    "..#.#.#..",
    ".#..#..#.",
    "....#....",
    "....#....",
];

/// `rows` as the record `unavi:vui` reads.
#[must_use]
pub fn from_rows(rows: &[&str]) -> Glyph {
    let (size, bits) = pack(rows);
    Glyph { size, bits }
}

/// A tool's announced mark as VUI's glyph. The two records are the same shape
/// but belong to different packages, and the tool package has already checked
/// the size and the bytes.
#[must_use]
pub fn from_tool(glyph: tool::Glyph) -> Glyph {
    Glyph {
        size: glyph.size,
        bits: glyph.bits,
    }
}

#[cfg(test)]
mod tests {
    use wired_guest::glyph::centred;

    use super::*;

    /// Pins the bit order and row stride for every bitmap at once: they all go
    /// through the same packer, so one exact picture is enough.
    #[test]
    fn home_packs_to_its_exact_bytes() {
        assert_eq!(
            from_rows(&HOME).bits,
            [
                0x00, 0x20, 0xE0, 0xE0, 0xE3, 0x8F, 0x0D, 0x1B, 0x36, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn every_bitmap_is_centred_in_its_grid() {
        for (name, rows) in [
            ("HOME", HOME),
            ("NAV", NAV),
            ("TOOLS", TOOLS),
            ("SELF", SELF),
            ("VISOR", VISOR),
            ("LENS_TRAVEL", LENS_TRAVEL),
            ("BEACON", BEACON),
            ("HERE", HERE),
            ("TRAVEL", TRAVEL),
            ("TOOL", TOOL),
        ] {
            assert!(centred(&rows), "{name} slides to one side of its orb");
        }
    }

    #[test]
    fn every_bitmap_is_nine_square() {
        for rows in [
            HOME,
            NAV,
            TOOLS,
            SELF,
            VISOR,
            LENS_TRAVEL,
            BEACON,
            HERE,
            TRAVEL,
            TOOL,
        ] {
            assert!(rows.iter().all(|row| row.len() == 9));
            let glyph = from_rows(&rows);
            assert_eq!((glyph.size, glyph.bits.len()), (9, 11));
        }
    }

    #[test]
    fn a_tool_glyph_carries_over_unchanged() {
        let glyph = from_tool(tool::Glyph {
            size: 3,
            bits: vec![5, 1],
        });
        assert_eq!((glyph.size, glyph.bits), (3, vec![5, 1]));
    }
}
