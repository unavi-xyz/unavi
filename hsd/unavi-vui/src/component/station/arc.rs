//! A station standing on a cylinder around the point it was planted, each
//! pane at reading distance and facing the axis.

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use super::tiling::{
    Cells,
    Slot,
};
use crate::{
    component::pane::{
        COLUMN,
        LINE,
    },
    tokens::tuning::Tuning,
};

/// The cylinder, in metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc {
    reach:  f32,
    /// The band a pane's middle may stand in, off the eye.
    low:    f32,
    high:   f32,
    /// Metres across one column and down one line.
    column: f32,
    line:   f32,
}

impl Arc {
    #[must_use]
    pub fn of(tuning: &Tuning) -> Self {
        Self {
            reach:  tuning.station_reach,
            low:    tuning.land_low,
            high:   tuning.land_high,
            column: COLUMN * tuning.pane_em,
            line:   LINE * tuning.pane_em,
        }
    }

    /// Where the middle of each slot of `region` stands, in the frame of the
    /// eye the station was planted at: ahead is `-Z`. A slot left of the
    /// region's middle stands left, and one lower stands lower.
    ///
    /// The whole station moves up or down together to sit in the band
    /// between `land_low` and `land_high`, which keeps panes from collapsing
    /// onto one height; one taller than the band is centred in it.
    #[must_use]
    pub fn poses(&self, slots: &[Slot], region: Cells) -> Vec<Transform> {
        let middle = self.low.midpoint(self.high);
        let heights = slots
            .iter()
            .map(|slot| (region.h as f32 / 2.0 - centre(slot.y, slot.h)) * self.line)
            .collect::<Vec<_>>();
        let lowest = heights.iter().copied().fold(f32::MAX, f32::min);
        let highest = heights.iter().copied().fold(f32::MIN, f32::max);
        let lift = if highest - lowest <= self.high - self.low {
            middle.clamp(self.low - lowest, self.high - highest)
        } else {
            middle - lowest.midpoint(highest)
        };

        slots
            .iter()
            .zip(heights)
            .map(|(slot, height)| {
                let across = (centre(slot.x, slot.w) - region.w as f32 / 2.0) * self.column;
                let angle = across / self.reach;
                Transform {
                    translation: Vec3::new(
                        self.reach * angle.sin(),
                        height + lift,
                        -self.reach * angle.cos(),
                    ),
                    rotation:    Quat::new(0.0, (-angle / 2.0).sin(), 0.0, (-angle / 2.0).cos()),
                    scale:       Vec3::ONE,
                }
            })
            .collect()
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

    fn slots() -> (Vec<Slot>, Cells) {
        let tiling = Tiling::new(Node::Split {
            axis: Axis::Down,
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
                                weight: 2,
                                node:   Node::Tabs(vec![2]),
                            },
                            Part {
                                weight: 1,
                                node:   Node::Tabs(vec![3]),
                            },
                        ],
                    },
                },
                Part {
                    weight: 1,
                    node:   Node::Tabs(vec![4]),
                },
            ],
        })
        .unwrap();
        let region = Cells { w: 160, h: 80 };
        (
            tiling.leaves(region).into_iter().map(|leaf| leaf.slot).collect(),
            region,
        )
    }

    fn arc() -> Arc {
        Arc::of(&Tuning::DEFAULT)
    }

    #[test]
    fn left_splits_stand_left_of_centre_and_bottom_splits_below() {
        let (slots, region) = slots();
        let poses = arc().poses(&slots, region);
        let [left, middle, right, bottom] = [&poses[0], &poses[1], &poses[2], &poses[3]];
        assert!(left.translation.x < -0.1);
        assert!(middle.translation.x.abs() < 1.0e-4);
        assert!(right.translation.x > 0.1);
        assert!(bottom.translation.y < middle.translation.y);
    }

    #[test]
    fn every_pane_is_at_reading_distance_from_the_axis() {
        let (slots, region) = slots();
        let reach = Tuning::DEFAULT.station_reach;
        for pose in arc().poses(&slots, region) {
            let flat = (pose.translation.x.powi(2) + pose.translation.z.powi(2)).sqrt();
            assert!((flat - reach).abs() < 1.0e-4, "{flat} is not {reach}");
            assert!(pose.translation.z < 0.0, "ahead of the planted point");
        }
    }

    #[test]
    fn every_pane_faces_the_axis() {
        let (slots, region) = slots();
        for pose in arc().poses(&slots, region) {
            let front = pose.rotation * Vec3::Z;
            let to_axis = Vec3::new(-pose.translation.x, 0.0, -pose.translation.z).normalize();
            assert!((front - to_axis).length() < 1.0e-4, "{front} is not {to_axis}");
        }
    }

    #[test]
    fn the_station_stands_in_the_band_below_the_eye() {
        let slots = [
            Slot {
                x: 0,
                y: 0,
                w: 40,
                h: 16,
            },
            Slot {
                x: 40,
                y: 0,
                w: 40,
                h: 16,
            },
        ];
        let region = Cells { w: 80, h: 16 };
        let tuning = Tuning::DEFAULT;
        for pose in arc().poses(&slots, region) {
            let y = pose.translation.y;
            assert!(
                (tuning.land_low..=tuning.land_high).contains(&y),
                "{y} is outside the band"
            );
        }
    }

    #[test]
    fn a_station_taller_than_the_band_is_centred_in_it() {
        let slots = [
            Slot { x: 0, y: 0, w: 40, h: 40 },
            Slot { x: 0, y: 40, w: 40, h: 40 },
        ];
        let region = Cells { w: 40, h: 400 };
        let tuning = Tuning::DEFAULT;
        let poses = arc().poses(&slots, region);
        let mean = (poses[0].translation.y + poses[1].translation.y) / 2.0;
        assert!((mean - (tuning.land_low + tuning.land_high) / 2.0).abs() < 1.0e-4);
        assert!(poses[0].translation.y > poses[1].translation.y);
    }
}
