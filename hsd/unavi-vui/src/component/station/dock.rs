//! A station laid against the edges of the local screen, in whole pixels.

use wired_guest::math::{
    Transform,
    Vec3,
};

use super::tiling::{
    Cells,
    Slot,
};

/// A rectangle of pixels from the screen's top left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixels {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Pixels across one column and down one line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grain {
    pub column: u32,
    pub line:   u32,
}

/// A screen a station can lay panes against: its size in pixels and the
/// vertical field of view it is drawn with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Screen {
    pub pixels: Cells,
    pub fov:    f32,
}

/// Where a pane of `size` cells goes, for the leaf `slot` of `region`, on a
/// screen of `view` pixels.
///
/// A leaf in the left half of the region goes against the left edge and one
/// in the right half against the right; a leaf that spans the middle is
/// centred. The same holds up and down. Everything else, the middle of the
/// screen, is left to the world.
#[must_use]
pub fn dock(slot: Slot, size: Cells, region: Cells, view: Cells, grain: Grain) -> Pixels {
    let (x, w) = along(slot.x, slot.w, size.w, region.w, view.w, grain.column);
    let (y, h) = along(slot.y, slot.h, size.h, region.h, view.h, grain.line);
    Pixels { x, y, w, h }
}

/// One axis, in pixels, with every length in cells but `grain` and the
/// results.
fn along(start: u32, run: u32, size: u32, region: u32, view: u32, grain: u32) -> (u32, u32) {
    let span = size * grain;
    let free = (view * grain).saturating_sub(span);
    let at = if start * 2 < region && region < (start + run) * 2 {
        free / 2
    } else if start * 2 + run <= region {
        start * grain
    } else {
        let beyond = region.saturating_sub(start + run).min(region);
        (view * grain).saturating_sub((beyond + size) * grain)
    };
    (at.min(free), span)
}

/// The depth at which one line lands on a whole number of pixels, and the
/// grain that makes it so, given one column and one line in metres.
///
/// A dot of a rule is one line across, so a line that is not a whole number of
/// pixels blurs it. The station therefore stands a little nearer or further
/// than `reading`, whichever puts a line on a whole number: at most half a
/// line's worth of pixels away from it.
#[must_use]
pub fn grain(column: f32, line: f32, screen: &Screen, reading: f32) -> (f32, Grain) {
    let half = (screen.fov / 2.0).tan();
    let pixels = (screen.pixels.h.max(1)) as f32;
    // Metres one pixel covers at the reading distance, then the whole number
    // of pixels a line is worth there, and the depth that makes it exact.
    let per_pixel = 2.0 * reading * half / pixels;
    let lines = (line / per_pixel).round().max(1.0);
    let snapped = line / lines;
    let depth = snapped * pixels / (2.0 * half);
    // A column is no taller than a line, so it may not land on a whole pixel
    // at the same depth; it is rounded to the nearest, and at least one.
    let columns = (column / snapped).round().max(1.0);
    (
        depth,
        Grain {
            column: columns as u32,
            line:   lines as u32,
        },
    )
}

/// Where a leaf stands, in the eye's frame: at `depth` ahead, its pixel
/// rectangle's middle mapped to metres, facing as the eye faces.
#[must_use]
pub fn place(rect: Pixels, screen: &Screen, depth: f32, eye: &Transform) -> Transform {
    let pixels = Cells {
        w: screen.pixels.w,
        h: screen.pixels.h,
    };
    let metres = depth * (screen.fov / 2.0).tan() * 2.0 / (pixels.h.max(1)) as f32;
    let across = (centre(rect.x, rect.w) - pixels.w as f32 / 2.0) * metres;
    let down = (pixels.h as f32 / 2.0 - centre(rect.y, rect.h)) * metres;
    Transform {
        translation: eye.translation + eye.rotation * Vec3::new(across, down, -depth),
        rotation:    eye.rotation,
        scale:       Vec3::ONE,
    }
}

fn centre(start: u32, run: u32) -> f32 {
    start as f32 + run as f32 / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::station::tiling::{
        Axis,
        Node,
        Part,
        Tiling,
    };

    const GRAIN: Grain = Grain { column: 8, line: 16 };
    const VIEW: Cells = Cells { w: 240, h: 67 };

    /// Hierarchy at the left, properties at the right, and a bar across the
    /// bottom.
    fn editor() -> (Vec<(u32, Slot, Cells)>, Cells) {
        let tiling = Tiling::new(Node::Split {
            axis:  Axis::Down,
            parts: vec![
                Part {
                    weight: 3,
                    node:   Node::Split {
                        axis:  Axis::Across,
                        parts: vec![
                            Part {
                                weight: 1,
                                node:   Node::Tabs(vec![1_u32]),
                            },
                            Part {
                                weight: 1,
                                node:   Node::Tabs(vec![2]),
                            },
                        ],
                    },
                },
                Part {
                    weight: 1,
                    node:   Node::Tabs(vec![3]),
                },
            ],
        })
        .unwrap();
        let size = |item: &u32| match item {
            1 | 2 => Cells { w: 40, h: 40 },
            _ => Cells { w: 80, h: 12 },
        };
        let region = tiling.natural(size);
        let leaves = tiling
            .leaves(region)
            .into_iter()
            .map(|leaf| (leaf.tabs[0], leaf.slot, size(&leaf.tabs[0])))
            .collect();
        (leaves, region)
    }

    fn docked() -> Vec<Pixels> {
        let (leaves, region) = editor();
        leaves
            .into_iter()
            .map(|(_, slot, size)| dock(slot, size, region, VIEW, GRAIN))
            .collect()
    }

    #[test]
    fn a_left_leaf_sits_on_the_left_edge_and_a_right_leaf_on_the_right() {
        let docked = docked();
        assert_eq!(docked[0].x, 0);
        assert_eq!(docked[1].x + docked[1].w, VIEW.w * GRAIN.column);
    }

    #[test]
    fn a_leaf_across_the_bottom_sits_on_the_bottom_edge_centred() {
        let docked = docked();
        assert_eq!(docked[2].y + docked[2].h, VIEW.h * GRAIN.line);
        assert_eq!(
            docked[2].x * 2 + docked[2].w,
            VIEW.w * GRAIN.column,
            "centred across"
        );
    }

    #[test]
    fn the_middle_of_the_screen_is_left_open() {
        let docked = docked();
        let width = VIEW.w * GRAIN.column;
        let height = VIEW.h * GRAIN.line;
        let (cx, cy) = (width / 2, height / 3);
        for pane in &docked {
            let inside = pane.x < cx && cx < pane.x + pane.w && pane.y < cy && cy < pane.y + pane.h;
            assert!(!inside, "{pane:?} covers the middle");
        }
    }

    #[test]
    fn every_edge_falls_on_a_whole_cell() {
        for pane in docked() {
            assert_eq!(pane.w % GRAIN.column, 0);
            assert_eq!(pane.h % GRAIN.line, 0);
        }
    }

    #[test]
    fn a_screen_too_small_keeps_the_pane_at_the_edge() {
        let slot = Slot {
            x: 0,
            y: 0,
            w: 40,
            h: 40,
        };
        let size = Cells { w: 40, h: 40 };
        let tiny = Cells { w: 10, h: 10 };
        let at = dock(slot, size, Cells { w: 80, h: 40 }, tiny, GRAIN);
        assert_eq!((at.x, at.y), (0, 0));
    }

    fn screen() -> Screen {
        Screen {
            pixels: Cells { w: 2560, h: 1440 },
            fov:    1.0,
        }
    }

    #[test]
    fn a_line_lands_on_a_whole_number_of_pixels() {
        let (depth, grain) = grain(0.0102, 0.0255, &screen(), 0.85);
        let per_pixel = 2.0 * depth * (screen().fov / 2.0).tan() / 1440.0;
        assert!((0.0255 / per_pixel - grain.line as f32).abs() < 1.0e-3);
        assert!(grain.line >= 1);
    }

    #[test]
    fn snapping_moves_the_station_by_less_than_a_line() {
        let (depth, _) = grain(0.0102, 0.0255, &screen(), 0.85);
        assert!((depth - 0.85).abs() < 0.05, "{depth} is not near the reading distance");
    }

    #[test]
    fn the_top_left_leaf_stands_up_and_to_the_left() {
        let docked = docked();
        let eye = Transform {
            translation: Vec3::new(0.0, 1.6, 2.0),
            rotation:    wired_guest::math::Quat::IDENTITY,
            scale:       Vec3::ONE,
        };
        let screen = screen();
        let (depth, _) = grain(0.0102, 0.0255, &screen, 0.85);
        let left = place(docked[0], &screen, depth, &eye);
        assert!(left.translation.x < eye.translation.x, "to the left");
        assert!(left.translation.y > eye.translation.y, "above the eye");
        assert!((left.translation.z - (eye.translation.z - depth)).abs() < 1.0e-4);
    }
}
