//! Dot glyphs authored as rows of text.
//!
//! Every script that announces a dot glyph, whether to `unavi:vui` or to
//! `unavi:tool`, authors it as a picture a person can read and edit, and both
//! packages ask for the same packed bytes. The packing lives here so each
//! script does not carry its own copy.

/// Packs `rows` into a square dot grid: returns its side and its bytes.
///
/// The side is the number of rows. Cells run row-major from the top-left, one
/// bit each, least significant bit first within a byte. `#` is a lit dot and
/// anything else is not; a row longer than the side is cut and a shorter one
/// leaves its remaining cells unlit.
#[must_use]
pub fn pack(rows: &[&str]) -> (u8, Vec<u8>) {
    // Both packages refuse a side above 13, so a taller picture is an authoring
    // mistake worth catching here rather than as a refusal at the far end.
    debug_assert!(rows.len() <= 13, "a glyph is at most 13 rows");
    let size = rows.len();
    let mut bits = vec![0_u8; (size * size).div_ceil(8)];
    for (row, line) in rows.iter().enumerate() {
        for (column, cell) in line.bytes().take(size).enumerate() {
            if cell == b'#' {
                let index = row * size + column;
                bits[index / 8] |= 1 << (index % 8);
            }
        }
    }
    (size as u8, bits)
}

/// Whether the lit cells of `rows` sit centred in their grid: the bounding box
/// of the `#` cells has equal margins on the left and right, and on the top and
/// bottom. A glyph with nothing lit counts as centred.
///
/// A mark that is off-centre by a cell reads as sliding to one side of the
/// shell it is drawn on, so authored bitmaps are worth checking with this.
#[must_use]
pub fn centred(rows: &[&str]) -> bool {
    let size = rows.len();
    let lit = rows.iter().enumerate().flat_map(|(row, line)| {
        line.bytes()
            .take(size)
            .enumerate()
            .filter(|(_, cell)| *cell == b'#')
            .map(move |(column, _)| (column, row))
    });
    let bounds = lit.fold(None, |bounds, (column, row)| {
        let (low, high) = bounds.unwrap_or(((column, row), (column, row)));
        Some((
            (low.0.min(column), low.1.min(row)),
            (high.0.max(column), high.1.max(row)),
        ))
    });
    bounds.is_none_or(|(low, high)| low.0 + high.0 == size - 1 && low.1 + high.1 == size - 1)
}

/// A person, head over shoulders, seven dots to a side: the mark for how
/// many people are somewhere. Every lit dot lies inside the circle a glyph is
/// clipped to.
pub const PERSON: [&str; 7] = [
    "..###..", "..###..", "..###..", ".......", ".#####.", ".#####.", "..###..",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_glyph_is_centred_when_its_lit_box_has_equal_margins() {
        assert!(centred(&["...", ".#.", "..."]));
        assert!(centred(&["#.#", "...", "#.#"]));
        assert!(centred(&["...", "...", "..."]), "nothing lit is centred");
    }

    #[test]
    fn a_glyph_slid_along_either_axis_is_not_centred() {
        assert!(!centred(&["#..", "...", "..."]), "to the corner");
        assert!(!centred(&["...", "##.", "..."]), "to the left");
        assert!(!centred(&["...", "...", ".#."]), "to the bottom");
        assert!(
            !centred(&[".....", ".###.", ".....", ".....", "....."]),
            "up, though level across"
        );
    }

    fn lit(size: u8, bits: &[u8], row: usize, column: usize) -> bool {
        let index = row * usize::from(size) + column;
        bits[index / 8] >> (index % 8) & 1 == 1
    }

    #[test]
    fn the_side_is_the_number_of_rows() {
        let (size, bits) = pack(&["........."; 9]);
        assert_eq!(size, 9);
        assert_eq!(bits.len(), 11, "81 cells need 11 bytes");
    }

    #[test]
    fn the_person_sits_centred_and_inside_the_clip() {
        assert!(centred(&PERSON));
        let reach = (PERSON.len() as f32 - 1.0) / 2.0 + 0.6;
        for (row, line) in PERSON.iter().enumerate() {
            for (column, cell) in line.bytes().enumerate() {
                let apart = (column as f32 - 3.0).hypot(row as f32 - 3.0);
                assert!(
                    cell != b'#' || apart <= reach,
                    "dot {column},{row} is clipped"
                );
            }
        }
    }

    #[test]
    fn bits_run_least_significant_first() {
        let (_, bits) = pack(&["#.#", "...", "..#"]);
        assert_eq!(bits, vec![0b0000_0101, 0b0000_0001]);
    }

    /// Row 5, column 2 of a nine-square grid is cell 47, the last bit of byte
    /// 5, which is where a wrong row stride or byte split shows.
    #[test]
    fn a_cell_late_in_the_grid_lands_in_its_byte() {
        let mut rows = ["........."; 9];
        rows[5] = "..#......";
        let (size, bits) = pack(&rows);
        assert!(lit(size, &bits, 5, 2));
        assert_eq!(bits[5] >> 7, 1);
        assert_eq!(bits.iter().map(|b| b.count_ones()).sum::<u32>(), 1);
    }

    #[test]
    fn short_and_long_rows_do_not_spill_into_their_neighbours() {
        let (size, bits) = pack(&["#####", "#", "", "", ""]);
        assert!((0..5).all(|column| lit(size, &bits, 0, column)));
        assert!(lit(size, &bits, 1, 0));
        assert!((1..5).all(|column| !lit(size, &bits, 1, column)));
    }
}
