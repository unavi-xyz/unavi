//! A mote's face as data: a small square grid of dots, the way a dot-matrix
//! display is addressed.
//!
//! The grid is checked and packed here, on the host-testable side, so the
//! shader only has to read bits back out of floats and the renderer only has
//! to compare two glyphs to know whether a face changed.

use std::fmt;

/// The largest grid a glyph may be. The shader's lane budget is sized for
/// this, so a bigger grid has nowhere to ride.
pub const MAX_SIZE: usize = 13;

/// Bits one float carries exactly. A float's mantissa holds 24 bits, so every
/// whole number up to 2^24 survives the trip to the shader unchanged.
pub const LANE_BITS: usize = 24;

/// The vec3 inputs a glyph's bits are spread over, three lanes each.
pub const LANE_TRIPLES: usize = (MAX_SIZE * MAX_SIZE).div_ceil(LANE_BITS * 3);

/// A glyph's bits as the shader receives them.
pub type Lanes = [[f32; 3]; LANE_TRIPLES];

/// Why a glyph was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// A grid with no cells, or more than [`MAX_SIZE`] to a side.
    Size(u8),
    /// Fewer bytes than the grid has cells for.
    Short { have: usize, need: usize },
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Size(size) => write!(f, "glyph size is {size}, not in 1..={MAX_SIZE}"),
            Self::Short { have, need } => {
                write!(f, "glyph has {have} bytes of bits, needs {need}")
            }
        }
    }
}

/// A square grid of dots, lit or not.
///
/// Only the cells inside the circle are kept: a cell [`inside`] never draws,
/// so whatever a caller put there is dropped rather than compared, and two
/// glyphs that look the same are equal.
#[derive(Debug, Clone, PartialEq)]
pub struct Glyph {
    size:  usize,
    lanes: Lanes,
}

impl Glyph {
    /// A glyph from the wire's shape: `size` cells to a side, bit `i` in byte
    /// `i / 8` counting from the least significant, cells in rows from the top
    /// left. Bytes past the grid are ignored.
    pub fn new(size: u8, bits: &[u8]) -> Result<Self, Refused> {
        let edge = usize::from(size);
        if edge == 0 || edge > MAX_SIZE {
            return Err(Refused::Size(size));
        }
        let need = (edge * edge).div_ceil(8);
        if bits.len() < need {
            return Err(Refused::Short {
                have: bits.len(),
                need,
            });
        }

        let mut lanes: Lanes = [[0.0; 3]; LANE_TRIPLES];
        for cell in 0..edge * edge {
            if is_set(bits, cell) && inside(edge, cell % edge, cell / edge) {
                let lane = cell / LANE_BITS;
                // Exact: each lane holds under 2^24, which a float keeps whole.
                lanes[lane / 3][lane % 3] += (1_u32 << (cell % LANE_BITS)) as f32;
            }
        }

        Ok(Self { size: edge, lanes })
    }

    #[must_use]
    pub const fn size(&self) -> usize {
        self.size
    }

    /// The bits spread over the shader's lanes, 24 to a float.
    #[must_use]
    pub const fn lanes(&self) -> &Lanes {
        &self.lanes
    }
}

/// Bit `cell` of `bits`, counting from the least significant bit of byte 0.
const fn is_set(bits: &[u8], cell: usize) -> bool {
    (bits[cell / 8] >> (cell % 8)) & 1 == 1
}

/// Whether the dot at `column`, `row` of a `size` grid is drawn at all.
///
/// A glyph is clipped to a circle a little wider than the one through its
/// outermost cell centres, so the corners of a square grid fall away and a
/// face reads as round, like the shell it sits in. The shader applies the same
/// rule to the same numbers.
#[must_use]
pub fn inside(size: usize, column: usize, row: usize) -> bool {
    let centre = (size as f32 - 1.0) * 0.5;
    let across = column as f32 - centre;
    let down = row as f32 - centre;
    across.hypot(down) <= centre + CLIP_MARGIN
}

/// How far past the outermost cell centres the clip circle reaches, in cells.
pub const CLIP_MARGIN: f32 = 0.6;

#[cfg(test)]
mod tests {
    use super::*;

    /// A glyph lighting exactly `cells` of a `size` grid, given as
    /// `(column, row)` and wire-packed the way a guest would.
    fn lighting(size: u8, cells: &[(usize, usize)]) -> Glyph {
        let edge = usize::from(size);
        let mut bits = vec![0_u8; (edge * edge).div_ceil(8)];
        for (column, row) in cells {
            let cell = row * edge + column;
            bits[cell / 8] |= 1 << (cell % 8);
        }
        Glyph::new(size, &bits).expect("a well-formed glyph")
    }

    /// Reads cell `cell` back out of `lanes` with the arithmetic the shader
    /// uses: scale by a power of two, drop the fraction, take the low bit.
    fn read(lanes: &Lanes, cell: usize) -> bool {
        let lane = cell / LANE_BITS;
        let value = lanes[lane / 3][lane % 3];
        let scaled = (value / (1_u32 << (cell % LANE_BITS)) as f32).floor();
        scaled % 2.0 > 0.5
    }

    /// Whether the reference says `cell` of a `size` grid shows, given the
    /// cells a caller lit.
    fn drawn(size: usize, lit: &[(usize, usize)], cell: usize) -> bool {
        lit.contains(&(cell % size, cell / size)) && inside(size, cell % size, cell / size)
    }

    fn clipped_count(size: usize) -> usize {
        (0..size * size)
            .filter(|cell| inside(size, cell % size, cell / size))
            .count()
    }

    #[test]
    fn the_circle_keeps_sixty_nine_cells_of_a_nine_grid() {
        assert_eq!(clipped_count(9), 69);
        assert!(inside(9, 4, 4));
        assert!(!inside(9, 0, 0), "a corner is outside the circle");
        assert!(inside(9, 4, 0), "the middle of an edge is not");
    }

    #[test]
    fn a_thirteen_grid_is_clipped_too() {
        let count = clipped_count(13);
        assert!(count < 169 && count > 100, "{count} cells survive");
    }

    #[test]
    fn bits_run_along_rows_from_the_top_left_and_from_the_low_bit() {
        // A three grid keeps every cell, so nothing here is clipped away.
        let first = Glyph::new(3, &[0b0000_0010, 0b0000_0000]).expect("a three grid");
        assert!(
            read(first.lanes(), 1),
            "bit 1 of byte 0 is the second cell of row 0"
        );
        assert!(!read(first.lanes(), 0));
        let second_byte = Glyph::new(3, &[0, 0b0000_0001]).expect("a three grid");
        assert!(
            read(second_byte.lanes(), 8),
            "bit 0 of byte 1 is cell 8, the last of a three grid"
        );
        assert!(!read(second_byte.lanes(), 7));
    }

    #[test]
    fn a_cell_in_a_clipped_corner_is_dropped() {
        let glyph = lighting(9, &[(0, 0), (4, 4)]);
        assert!(!read(glyph.lanes(), 0));
        assert!(read(glyph.lanes(), 4 * 9 + 4));
        assert_eq!(glyph, lighting(9, &[(4, 4)]), "equal by what is drawn");
    }

    #[test]
    fn packing_round_trips_a_nine_grid() {
        let cells = (0..9)
            .flat_map(|row| (0..9).map(move |column| (column, row)))
            .filter(|(column, row)| (column * 3 + row * 5) % 4 == 0)
            .collect::<Vec<_>>();
        let glyph = lighting(9, &cells);
        for cell in 0..81 {
            assert_eq!(
                read(glyph.lanes(), cell),
                drawn(9, &cells, cell),
                "cell {cell}"
            );
        }
    }

    #[test]
    fn packing_round_trips_a_thirteen_grid_through_the_last_lane() {
        let cells = (0..13)
            .flat_map(|row| (0..13).map(move |column| (column, row)))
            .filter(|(column, row)| (column + row) % 2 == 0)
            .collect::<Vec<_>>();
        let glyph = lighting(13, &cells);
        let lit = (0..169).filter(|cell| read(glyph.lanes(), *cell)).count();
        assert!(lit > 40, "a checkerboard is half lit, minus its corners");
        for cell in 0..169 {
            assert_eq!(
                read(glyph.lanes(), cell),
                drawn(13, &cells, cell),
                "cell {cell}"
            );
        }
    }

    #[test]
    fn the_last_cell_that_survives_the_clip_rides_the_seventh_lane() {
        // The final cell of a thirteen grid is a clipped corner, so the
        // eighth lane is never needed: the last drawable cell is in the
        // seventh, the first of the third triple.
        let cell = (0..169)
            .rev()
            .find(|cell| inside(13, cell % 13, cell / 13))
            .expect("some cell survives");
        assert!((6 * LANE_BITS..7 * LANE_BITS).contains(&cell));
        let glyph = lighting(13, &[(cell % 13, cell / 13)]);
        assert!(glyph.lanes()[2][0] > 0.0);
        assert!(read(glyph.lanes(), cell));
    }

    #[test]
    fn a_lane_never_holds_more_than_a_float_keeps_whole() {
        let full = vec![0xFF_u8; 22];
        let glyph = Glyph::new(13, &full).expect("a full thirteen grid");
        for lane in glyph.lanes().iter().flatten() {
            assert!(*lane < (1_u32 << LANE_BITS) as f32);
            assert_eq!(lane.fract(), 0.0);
        }
    }

    #[test]
    fn a_grid_of_no_size_or_too_big_a_size_is_refused() {
        assert_eq!(Glyph::new(0, &[]), Err(Refused::Size(0)));
        assert_eq!(Glyph::new(14, &[0; 64]), Err(Refused::Size(14)));
        assert!(Glyph::new(13, &[0; 22]).is_ok());
    }

    #[test]
    fn too_few_bytes_is_refused_and_too_many_is_not() {
        assert_eq!(
            Glyph::new(9, &[0; 10]),
            Err(Refused::Short { have: 10, need: 11 })
        );
        assert!(Glyph::new(9, &[0; 40]).is_ok());
    }
}
