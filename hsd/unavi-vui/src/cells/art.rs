//! Pictures as cells: a bitmap taken as it is, a greyscale image through
//! ordered 4×4 Bayer.

use super::{
    Grid,
    Level,
    Refused,
};

/// The 4×4 Bayer index matrix, in rows. The website dithers with the same
/// sixteen numbers, so a picture reads the same in both places.
pub const BAYER: [u8; 16] = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

impl Grid {
    /// A bitmap, `width` wide: set bits lit, the rest off.
    pub fn from_bitmap(width: usize, bits: &[bool]) -> Result<Self, Refused> {
        let levels = bits
            .iter()
            .map(|set| if *set { Level::Lit } else { Level::Off })
            .collect::<Vec<_>>();
        Self::new(width, rows(width, levels.len()), levels)
    }

    /// A greyscale image, `width` wide, as two tones. A cell is lit where its
    /// grey, `0..=255`, is above the Bayer threshold at its place in the
    /// repeating 4×4 tile.
    pub fn dithered(width: usize, grey: &[u8]) -> Result<Self, Refused> {
        let levels = grey
            .iter()
            .enumerate()
            .map(|(cell, grey)| {
                let index = BAYER[(cell / width.max(1) % 4) * 4 + cell % width.max(1) % 4];
                // grey / 255 > (index + 0.5) / 16, without the division.
                if u32::from(*grey) * 32 > 255 * (2 * u32::from(index) + 1) {
                    Level::Lit
                } else {
                    Level::Off
                }
            })
            .collect::<Vec<_>>();
        Self::new(width, rows(width, levels.len()), levels)
    }
}

/// Rows in `cells` cells of `width`. A width of zero has none, which the
/// grid refuses as empty.
fn rows(width: usize, cells: usize) -> usize {
    cells.checked_div(width).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(grid: &Grid) -> usize {
        (0..grid.height())
            .flat_map(|row| (0..grid.width()).map(move |column| (column, row)))
            .filter(|(column, row)| grid.level(*column, *row) == Level::Lit)
            .count()
    }

    #[test]
    fn the_matrix_is_the_one_the_website_dithers_with() {
        let website = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];
        assert_eq!(BAYER, website);
    }

    #[test]
    fn a_flat_grey_lights_the_share_of_the_tile_its_level_says() {
        for (grey, expected) in [(0_u8, 0), (128, 8), (255, 16)] {
            let tile = Grid::dithered(4, &[grey; 16]).expect("a tile");
            assert_eq!(lit(&tile), expected, "grey {grey}");
        }
    }

    #[test]
    fn the_threshold_is_the_website_s_at_every_place_in_the_tile() {
        for place in 0..16 {
            let threshold = (f32::from(BAYER[place]) + 0.5) / 16.0;
            for grey in 0..=u8::MAX {
                let mut image = [0_u8; 16];
                image[place] = grey;
                let tile = Grid::dithered(4, &image).expect("a tile");
                let site = f32::from(grey) / 255.0 > threshold;
                assert_eq!(
                    tile.level(place % 4, place / 4) == Level::Lit,
                    site,
                    "place {place}, grey {grey}"
                );
            }
        }
    }

    #[test]
    fn the_tile_repeats_across_a_wider_image() {
        let grid = Grid::dithered(8, &[128; 64]).expect("an image");
        for row in 0..8 {
            for column in 0..8 {
                assert_eq!(
                    grid.level(column, row),
                    grid.level(column % 4, row % 4),
                    "{column},{row}"
                );
            }
        }
    }

    #[test]
    fn a_bitmap_lights_its_set_bits() {
        let grid = Grid::from_bitmap(2, &[true, false, false, true]).expect("a bitmap");
        assert_eq!((grid.width(), grid.height()), (2, 2));
        assert_eq!(grid.level(0, 0), Level::Lit);
        assert_eq!(grid.level(1, 0), Level::Off);
        assert_eq!(grid.level(1, 1), Level::Lit);
    }

    #[test]
    fn a_picture_with_no_width_or_a_ragged_last_row_is_refused() {
        assert_eq!(Grid::from_bitmap(0, &[true]), Err(Refused::Empty));
        assert_eq!(
            Grid::dithered(2, &[0, 0, 0]),
            Err(Refused::Length { have: 3, need: 2 })
        );
    }
}
