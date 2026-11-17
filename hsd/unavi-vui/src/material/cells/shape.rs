//! Shapes sampled onto cells.

use super::{
    Grid,
    Level,
    Refused,
};

impl Grid {
    /// A shape sampled at each cell's centre. `shape` takes a point in cells
    /// from the top left and returns its signed distance to the shape's
    /// edge in cells, negative inside. A cell inside is lit and one outside
    /// but within a cell of the edge is ember, which is how a thin stroke,
    /// narrower than a cell, still draws.
    pub fn sampled(
        width: usize,
        height: usize,
        shape: impl Fn(f32, f32) -> f32,
    ) -> Result<Self, Refused> {
        let mut levels = Vec::with_capacity(width.saturating_mul(height));
        for row in 0..height {
            for column in 0..width {
                let distance = shape(column as f32 + 0.5, row as f32 + 0.5);
                levels.push(if distance <= 0.0 {
                    Level::Lit
                } else if distance <= 1.0 {
                    Level::Ember
                } else {
                    Level::Off
                });
            }
        }
        Self::new(width, height, levels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(centre: f32, radius: f32) -> impl Fn(f32, f32) -> f32 {
        move |x, y| (x - centre).hypot(y - centre) - radius
    }

    #[test]
    fn inside_is_lit_the_edge_ring_is_ember_and_the_rest_is_off() {
        let grid = Grid::sampled(12, 12, circle(6.0, 3.0)).expect("a grid");
        assert_eq!(grid.level(5, 5), Level::Lit, "the middle");
        assert_eq!(
            grid.level(6, 3),
            Level::Lit,
            "a cell centre inside the edge"
        );
        assert_eq!(grid.level(0, 0), Level::Off, "a corner");
        assert_eq!(grid.level(6, 9), Level::Ember, "just outside the edge");
        assert_eq!(grid.level(6, 10), Level::Off, "a cell past the ember ring");
    }

    #[test]
    fn a_stroke_between_two_rows_reads_as_ember() {
        // A line a fifth of a cell wide lying between row 5 and row 6: no
        // cell centre is inside it, both neighbours are within a cell.
        let line = |_x: f32, y: f32| (y - 6.0).abs() - 0.1;
        let grid = Grid::sampled(12, 12, line).expect("a grid");
        for column in 0..12 {
            assert_eq!(grid.level(column, 5), Level::Ember);
            assert_eq!(grid.level(column, 6), Level::Ember);
            assert_eq!(grid.level(column, 4), Level::Off);
        }
    }

    #[test]
    fn a_stroke_through_cell_centres_is_lit_with_an_ember_edge() {
        let line = |_x: f32, y: f32| (y - 5.5).abs() - 0.2;
        let grid = Grid::sampled(12, 12, line).expect("a grid");
        assert_eq!(grid.level(3, 5), Level::Lit);
        assert_eq!(grid.level(3, 4), Level::Ember);
        assert_eq!(grid.level(3, 6), Level::Ember);
    }

    #[test]
    fn a_grid_with_no_cells_is_refused() {
        assert_eq!(Grid::sampled(0, 12, circle(0.0, 1.0)), Err(Refused::Empty));
    }
}
