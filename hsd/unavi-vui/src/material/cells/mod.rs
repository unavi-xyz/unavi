//! Cell art as data: a grid of three-level cells, one texel each.

pub mod art;
mod shape;

use std::fmt;

use wired_guest::style;

/// How lit one cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Off,
    Ember,
    Lit,
}

impl Level {
    /// The byte a texture holds for this level. The cell graph reads it back
    /// whether or not the host decodes the image as sRGB, so the ember byte
    /// sits where both encodings land between the two thresholds.
    #[must_use]
    pub const fn texel(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Ember => 128,
            Self::Lit => 255,
        }
    }
}

/// Why a grid was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// A grid with no cells.
    Empty,
    /// A different number of levels than the grid has cells.
    Length { have: usize, need: usize },
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Empty => write!(f, "cell grid has no cells"),
            Self::Length { have, need } => {
                write!(f, "cell grid has {have} levels, needs {need}")
            }
        }
    }
}

/// A rectangle of cells in rows from the top left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    width:  usize,
    height: usize,
    levels: Vec<Level>,
}

impl Grid {
    pub fn new(width: usize, height: usize, levels: Vec<Level>) -> Result<Self, Refused> {
        let need = width.saturating_mul(height);
        if need == 0 {
            return Err(Refused::Empty);
        }
        if levels.len() != need {
            return Err(Refused::Length {
                have: levels.len(),
                need,
            });
        }
        Ok(Self {
            width,
            height,
            levels,
        })
    }

    #[must_use]
    pub const fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// The level at `column`, `row`. A cell outside the grid is off.
    #[must_use]
    pub fn level(&self, column: usize, row: usize) -> Level {
        if column >= self.width || row >= self.height {
            return Level::Off;
        }
        self.levels[row * self.width + column]
    }

    /// One byte per cell, the one-channel image the cell graph samples.
    #[must_use]
    pub fn texels(&self) -> Vec<u8> {
        self.levels.iter().map(|level| level.texel()).collect()
    }

    /// The texels as RGBA8, grey and opaque, for a host that takes no
    /// one-channel format.
    #[must_use]
    pub fn rgba(&self) -> Vec<u8> {
        self.levels
            .iter()
            .flat_map(|level| {
                let texel = level.texel();
                [texel, texel, texel, u8::MAX]
            })
            .collect()
    }
}

/// How far a cell is flattened toward its average coverage at a pitch of
/// `pitch_px` screen pixels: `0` while the cell shape resolves, `1` once the
/// pitch is under [`style::CELLS_MOIRE_PX`]. The fade spans one more of those
/// widths, so a surface walking away turns to a grey picture rather than
/// shimmering.
#[must_use]
pub fn flatness(pitch_px: f32) -> f32 {
    let t = ((pitch_px - style::CELLS_MOIRE_PX) / style::CELLS_MOIRE_PX).clamp(0.0, 1.0);
    let eased = t * t * 2.0f32.mul_add(-t, 3.0);
    1.0 - eased
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grid_needs_cells_and_a_level_for_each() {
        assert_eq!(Grid::new(0, 4, Vec::new()), Err(Refused::Empty));
        assert_eq!(
            Grid::new(2, 2, vec![Level::Off; 3]),
            Err(Refused::Length { have: 3, need: 4 })
        );
    }

    #[test]
    fn a_cell_outside_the_grid_is_off() {
        let grid = Grid::new(2, 1, vec![Level::Lit, Level::Ember]).expect("a grid");
        assert_eq!(grid.level(1, 0), Level::Ember);
        assert_eq!(grid.level(2, 0), Level::Off);
        assert_eq!(grid.level(0, 1), Level::Off);
    }

    #[test]
    fn texels_and_rgba_carry_the_same_bytes() {
        let grid = Grid::new(3, 1, vec![Level::Off, Level::Ember, Level::Lit]).expect("a grid");
        assert_eq!(grid.texels(), [0, 128, 255]);
        assert_eq!(
            grid.rgba(),
            [0, 0, 0, 255, 128, 128, 128, 255, 255, 255, 255, 255]
        );
    }

    #[test]
    fn a_resolved_pitch_is_not_flattened_and_a_fine_one_is_flat() {
        let moire = style::CELLS_MOIRE_PX;
        assert!(flatness(moire * 2.0).abs() < f32::EPSILON);
        assert!(flatness(moire * 8.0).abs() < f32::EPSILON);
        assert!((flatness(moire) - 1.0).abs() < f32::EPSILON);
        assert!((flatness(0.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn flatness_falls_as_the_pitch_grows() {
        let moire = style::CELLS_MOIRE_PX;
        let mut last = 1.0;
        for step in 0..=20 {
            let now = flatness(moire + moire * step as f32 / 20.0);
            assert!(
                now <= last + f32::EPSILON,
                "step {step}: {now} after {last}"
            );
            last = now;
        }
    }
}
