//! Boxes around documents: measured in a document's own frame, posed into the
//! visor's, and cut into corner brackets.

use wired_guest::math::{
    Transform,
    Vec3,
};

/// An axis-aligned box. [`Aabb::EMPTY`] holds nothing and grows to the first
/// point given.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

/// One bracket bar: a segment along a box edge, in the frame the box was posed
/// into.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    pub centre: Vec3,
    /// The box axis it runs along: 0 for x, 1 for y, 2 for z.
    pub axis:   usize,
    /// Its length in metres, after the pose's scale.
    pub length: f32,
}

/// Bars in a box's brackets: three at each of eight corners.
pub const BARS: usize = 24;

impl Aabb {
    pub const EMPTY: Self = Self {
        min: Vec3::splat(f32::MAX),
        max: Vec3::splat(f32::MIN),
    };

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x
    }

    /// A box of these full extents centred on the origin.
    #[must_use]
    pub fn centred(extents: Vec3) -> Self {
        Self {
            min: -extents * 0.5,
            max: extents * 0.5,
        }
    }

    pub fn grow(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Grows by packed XYZ `positions`, each posed by `pose` first.
    pub fn grow_posed(&mut self, positions: &[f32], pose: &Transform) {
        for chunk in positions.as_chunks::<3>().0 {
            self.grow(pose.transform_point(Vec3::new(chunk[0], chunk[1], chunk[2])));
        }
    }

    pub fn union(&mut self, other: &Self) {
        if other.is_empty() {
            return;
        }
        self.grow(other.min);
        self.grow(other.max);
    }

    /// The box around this one's eight corners once posed by `pose`.
    #[must_use]
    pub fn posed(&self, pose: &Transform) -> Self {
        let mut out = Self::EMPTY;
        if self.is_empty() {
            return out;
        }
        for corner in self.corners() {
            out.grow(pose.transform_point(corner));
        }
        out
    }

    #[must_use]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    #[must_use]
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }

    /// The eight corners. Bit 0 of the index picks x, bit 1 y, bit 2 z, each
    /// the minimum when clear.
    #[must_use]
    pub fn corners(&self) -> [Vec3; 8] {
        std::array::from_fn(|index| corner(self, index))
    }
}

fn corner(aabb: &Aabb, index: usize) -> Vec3 {
    let pick = |bit: usize, min: f32, max: f32| if (index >> bit) & 1 == 1 { max } else { min };
    Vec3::new(
        pick(0, aabb.min.x, aabb.max.x),
        pick(1, aabb.min.y, aabb.max.y),
        pick(2, aabb.min.z, aabb.max.z),
    )
}

/// The extent of a capsule or cylinder along Y, with `radius` across.
#[must_use]
pub fn upright(half_height: f32, radius: f32) -> Aabb {
    Aabb {
        min: Vec3::new(-radius, -half_height, -radius),
        max: Vec3::new(radius, half_height, radius),
    }
}

/// The box a local `aabb` fills once its frame is posed by `root`, as its eight
/// corners, so the rotation is kept.
#[must_use]
pub fn world_corners(aabb: &Aabb, root: &Transform) -> [Vec3; 8] {
    aabb.corners().map(|point| root.transform_point(point))
}

/// The distance from `from` to the nearest corner of the posed box.
#[must_use]
pub fn nearest_corner(aabb: &Aabb, root: &Transform, from: Vec3) -> f32 {
    world_corners(aabb, root)
        .iter()
        .map(|corner| corner.distance(from))
        .fold(f32::MAX, f32::min)
}

/// Corner brackets for `aabb` posed by `root`: along each edge at each corner,
/// `fraction` of the edge, at most `cap` metres. An edge shorter than that is
/// drawn whole.
///
/// Bars lie along the box's own axes, so they all take `root`'s rotation. An
/// axis with no scale yields no length.
#[must_use]
pub fn brackets(aabb: &Aabb, root: &Transform, fraction: f32, cap: f32) -> [Bar; BARS] {
    let size = aabb.size();
    let scale = root.scale.abs();
    let mut bars = [Bar {
        centre: Vec3::ZERO,
        axis:   0,
        length: 0.0,
    }; BARS];
    for index in 0..8 {
        let at = corner(aabb, index);
        for axis in 0..3 {
            let extent = size[axis] * scale[axis];
            let length = (extent * fraction).min(cap).min(extent);
            let inward = if (index >> axis) & 1 == 1 { -1.0 } else { 1.0 };
            let reach = if scale[axis] > f32::EPSILON {
                inward * length / scale[axis] * 0.5
            } else {
                0.0
            };
            let mut centre = at;
            centre[axis] += reach;
            bars[index * 3 + axis] = Bar {
                centre: root.transform_point(centre),
                axis,
                length: if scale[axis] > f32::EPSILON {
                    length
                } else {
                    0.0
                },
            };
        }
    }
    bars
}

/// How far along the ray from `origin` along `direction` it enters the posed
/// box, within `reach`. A ray starting inside the box never enters it.
#[must_use]
pub fn enters(
    aabb: &Aabb,
    root: &Transform,
    origin: Vec3,
    direction: Vec3,
    reach: f32,
) -> Option<f32> {
    let inverse = root.inverse();
    let from = inverse.transform_point(origin);
    let along = inverse.rotation * direction * inverse.scale;
    let mut near = 0.0_f32;
    let mut far = reach;
    let mut inside = true;
    for axis in 0..3 {
        let (lo, hi) = (aabb.min[axis], aabb.max[axis]);
        if along[axis].abs() < f32::EPSILON {
            if from[axis] < lo || from[axis] > hi {
                return None;
            }
            continue;
        }
        if from[axis] < lo || from[axis] > hi {
            inside = false;
        }
        let (a, b) = (
            (lo - from[axis]) / along[axis],
            (hi - from[axis]) / along[axis],
        );
        near = near.max(a.min(b));
        far = far.min(a.max(b));
        if near > far {
            return None;
        }
    }
    (!inside).then_some(near)
}

/// The keys within `range` of the viewer, nearest first, at most `cap`.
#[must_use]
pub fn nearest<K: Copy>(candidates: &[(K, f32)], range: f32, cap: usize) -> Vec<K> {
    let mut near: Vec<_> = candidates
        .iter()
        .filter(|(_, distance)| *distance <= range)
        .collect();
    near.sort_by(|a, b| a.1.total_cmp(&b.1));
    near.into_iter().take(cap).map(|(key, _)| *key).collect()
}

#[cfg(test)]
mod tests {
    use wired_guest::math::Quat;

    use super::*;

    const UNIT: Aabb = Aabb {
        min: Vec3::ZERO,
        max: Vec3::ONE,
    };

    fn quarter_turn_about_y() -> Transform {
        Transform::new(
            Vec3::new(10.0, 0.0, 0.0),
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            Vec3::ONE,
        )
    }

    fn near(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1.0e-4
    }

    #[test]
    fn a_union_covers_both_boxes_and_ignores_an_empty_one() {
        let mut total = Aabb::EMPTY;
        total.union(&UNIT);
        total.union(&Aabb {
            min: Vec3::splat(-2.0),
            max: Vec3::splat(-1.0),
        });
        total.union(&Aabb::EMPTY);
        assert_eq!(total.min, Vec3::splat(-2.0));
        assert_eq!(total.max, Vec3::ONE);
        assert!(Aabb::EMPTY.is_empty());
    }

    #[test]
    fn posed_points_grow_the_box_in_the_new_frame() {
        let mut aabb = Aabb::EMPTY;
        let pose = Transform::new(Vec3::new(1.0, 0.0, 0.0), Quat::IDENTITY, Vec3::splat(2.0));
        aabb.grow_posed(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 9.0], &pose);
        assert_eq!(aabb.min, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(aabb.max, Vec3::new(3.0, 2.0, 2.0));
    }

    #[test]
    fn a_rotated_root_keeps_the_box_oriented() {
        let long = Aabb {
            min: Vec3::ZERO,
            max: Vec3::new(4.0, 1.0, 1.0),
        };
        let corners = world_corners(&long, &quarter_turn_about_y());
        assert!(near(corners[0], Vec3::new(10.0, 0.0, 0.0)));
        assert!(near(corners[1], Vec3::new(10.0, 0.0, -4.0)));
        assert!(near(corners[7], Vec3::new(11.0, 1.0, -4.0)));
    }

    #[test]
    fn a_collider_shape_box_is_centred() {
        let cuboid = Aabb::centred(Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(cuboid.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(upright(2.0, 0.5).size(), Vec3::new(1.0, 4.0, 1.0));
    }

    #[test]
    fn brackets_take_a_fraction_of_each_edge_up_to_the_cap() {
        let room = Aabb {
            min: Vec3::ZERO,
            max: Vec3::new(10.0, 1.0, 100.0),
        };
        let bars = brackets(&room, &Transform::IDENTITY, 0.15, 0.25);
        assert_eq!(bars.len(), BARS);
        assert!(bars.iter().all(|bar| bar.length <= 0.25 + 1.0e-6));
        let short = bars.iter().find(|bar| bar.axis == 1).map(|bar| bar.length);
        assert_eq!(short, Some(0.15));
    }

    #[test]
    fn an_edge_shorter_than_a_bracket_is_drawn_whole() {
        let tiny = Aabb {
            min: Vec3::ZERO,
            max: Vec3::new(0.1, 5.0, 5.0),
        };
        let bars = brackets(&tiny, &Transform::IDENTITY, 1.5, 3.0);
        let x_bar = bars.iter().find(|bar| bar.axis == 0).map(|bar| bar.length);
        assert_eq!(x_bar, Some(0.1));
    }

    #[test]
    fn bars_start_at_their_corner_and_run_inward() {
        let bars = brackets(&UNIT, &Transform::IDENTITY, 0.2, 1.0);
        assert!(near(bars[0].centre, Vec3::new(0.1, 0.0, 0.0)));
        // Corner 1 is the maximum x, so its x bar runs back toward the origin.
        assert!(near(bars[3].centre, Vec3::new(0.9, 0.0, 0.0)));
    }

    #[test]
    fn bars_follow_the_root_pose() {
        let bars = brackets(&UNIT, &quarter_turn_about_y(), 0.2, 1.0);
        // The local x axis points down -z once turned, so the first bar's
        // centre sits 0.1 along it from the root's origin.
        assert!(near(bars[0].centre, Vec3::new(10.0, 0.0, -0.1)));
    }

    #[test]
    fn a_scaled_root_scales_the_bar_length() {
        let root = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::splat(10.0));
        let bars = brackets(&UNIT, &root, 0.15, 0.25);
        assert!(bars.iter().all(|bar| (bar.length - 0.25).abs() < 1.0e-6));
        assert!(near(bars[0].centre, Vec3::new(0.125, 0.0, 0.0)));
    }

    #[test]
    fn a_ray_enters_a_box_ahead_of_it() {
        let hit = enters(
            &UNIT,
            &Transform::IDENTITY,
            Vec3::new(0.5, 0.5, 5.0),
            Vec3::NEG_Z,
            20.0,
        );
        assert!((hit.unwrap_or(0.0) - 4.0).abs() < 1.0e-5);
    }

    #[test]
    fn a_ray_that_misses_or_falls_short_enters_nothing() {
        let miss = Vec3::new(3.0, 0.5, 5.0);
        assert_eq!(
            enters(&UNIT, &Transform::IDENTITY, miss, Vec3::NEG_Z, 20.0),
            None
        );
        let short = Vec3::new(0.5, 0.5, 5.0);
        assert_eq!(
            enters(&UNIT, &Transform::IDENTITY, short, Vec3::NEG_Z, 3.0),
            None
        );
    }

    #[test]
    fn a_ray_from_inside_does_not_enter() {
        let inside = Vec3::splat(0.5);
        assert_eq!(
            enters(&UNIT, &Transform::IDENTITY, inside, Vec3::X, 20.0),
            None
        );
    }

    #[test]
    fn a_ray_is_tested_against_the_rotated_box() {
        let long = Aabb {
            min: Vec3::ZERO,
            max: Vec3::new(4.0, 1.0, 1.0),
        };
        let root = quarter_turn_about_y();
        // The box runs from z = 0 to -4 at x = 10..11 once turned.
        let through = Vec3::new(10.5, 0.5, -3.0);
        let hit = enters(&long, &root, through + Vec3::Y * 5.0, Vec3::NEG_Y, 20.0);
        assert!((hit.unwrap_or(0.0) - 4.5).abs() < 1.0e-4);
        let missed = Vec3::new(10.5, 0.5, 3.0);
        assert_eq!(
            enters(&long, &root, missed + Vec3::Y * 5.0, Vec3::NEG_Y, 20.0),
            None
        );
    }

    #[test]
    fn the_nearest_corner_is_measured_in_the_world() {
        let root = Transform::from_translation(Vec3::new(0.0, 0.0, 10.0));
        let distance = nearest_corner(&UNIT, &root, Vec3::new(0.0, 0.0, 7.0));
        assert!((distance - 3.0).abs() < 1.0e-5);
    }

    #[test]
    fn only_the_nearest_in_range_are_kept() {
        let all = [(1, 5.0), (2, 1.0), (3, 40.0), (4, 3.0)];
        assert_eq!(nearest(&all, 30.0, 2), vec![2, 4]);
    }
}
