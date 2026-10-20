//! Pulls a pane toward the viewer until nothing stands between them.

use unavi_lens::DocumentId;
use wired_guest::math::{
    Ray,
    Vec2,
    Vec3,
};

use crate::wired::{
    physics::simulation::{
        RayFilter,
        raycast,
    },
    scene::document::script_document,
};

/// The middle and four corners of a pane.
const PROBES: usize = 5;

/// Metres between a pane and the nearest thing in front of it.
const MARGIN: f32 = 0.02;

/// Metres from the eye a pane is never pulled closer than, so a pane is never
/// pushed into the viewer's face.
const NEAREST: f32 = 0.8;

/// Metres a pane cutting into something steps toward the eye at a time, and
/// the most steps it takes.
const SETTLE_STEP: f32 = 0.15;
const SETTLE_STEPS: usize = 6;

/// Metres a pulled position is rounded to, so small changes of eye do not
/// resend it.
const STEP: f32 = 0.005;

/// Casts from the eye to a pane and moves the pane in front of whatever the
/// rays hit.
pub struct Clearance {
    /// The visor's own document, whose colliders are never in the way.
    own:    DocumentId,
    /// The user's shell, whose surfaces float in front of the viewer and are
    /// never in the way either.
    shell:  Option<DocumentId>,
    warned: bool,
}

impl Clearance {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            own:    script_document()?.id(),
            shell:  None,
            warned: false,
        })
    }

    pub const fn set_shell(&mut self, shell: Option<DocumentId>) {
        self.shell = shell;
    }

    /// Where a `size` pane wanted with its bottom centre at `at` stands as
    /// seen from `eye`. Colliders in `exclude` do not count.
    pub fn clear(&mut self, eye: Vec3, at: Vec3, size: Vec2, exclude: &[DocumentId]) -> Vec3 {
        let probes = probes(eye, at, size);
        let mut exclude_documents = vec![self.own];
        exclude_documents.extend(self.shell);
        exclude_documents.extend_from_slice(exclude);
        let filter = RayFilter {
            exclude_local_agent: true,
            exclude_documents,
        };
        let mut hits = [None; PROBES];
        for (hit, probe) in hits.iter_mut().zip(&probes) {
            *hit = self.cast(eye, *probe, &filter);
        }
        let seen = pulled(eye, at, &probes, &hits);
        self.settle(eye, seen, size, &filter)
    }

    /// Steps a pane that cuts into something toward the eye until it no
    /// longer does. Rays from the eye miss a pane that slices into a wall at a
    /// slant, so this casts from the pane's middle out to its edges.
    fn settle(&mut self, eye: Vec3, at: Vec3, size: Vec2, filter: &RayFilter) -> Vec3 {
        let offset = at - eye;
        let length = offset.length();
        if length < f32::EPSILON {
            return at;
        }
        let floor = (NEAREST / length).min(1.0);
        let mut ratio = 1.0_f32;
        let mut placed = at;
        for _ in 0..SETTLE_STEPS {
            if !self.cuts(eye, placed, size, filter) || ratio <= floor {
                return placed;
            }
            ratio = (ratio - SETTLE_STEP / length).max(floor);
            placed = snap(eye + offset * ratio);
        }
        placed
    }

    /// Whether anything lies between the pane's middle and its edges.
    fn cuts(&mut self, eye: Vec3, at: Vec3, size: Vec2, filter: &RayFilter) -> bool {
        let [middle, corners @ ..] = probes(eye, at, size);
        let sides = [
            corners[0].midpoint(corners[2]),
            corners[1].midpoint(corners[3]),
        ];
        corners
            .into_iter()
            .chain(sides)
            .any(|edge| self.cast(middle, edge, filter).is_some())
    }

    /// The distance from `from` to the first collider on the way to `to`.
    fn cast(&mut self, from: Vec3, to: Vec3, filter: &RayFilter) -> Option<f32> {
        let reach = from.distance(to);
        if reach < f32::EPSILON {
            return None;
        }
        let ray = Ray {
            origin:    from,
            direction: (to - from) / reach,
        };
        match raycast(ray, reach, filter) {
            Ok(hit) => hit.map(|hit| hit.distance),
            Err(err) => {
                if !self.warned {
                    eprintln!("visor: clearance raycast: {err:?}");
                    self.warned = true;
                }
                None
            }
        }
    }
}

/// The middle and corners of a `size` pane with its bottom centre at `at`,
/// which faces `eye` about the vertical axis.
fn probes(eye: Vec3, at: Vec3, size: Vec2) -> [Vec3; PROBES] {
    let ahead = Vec3::new(at.x - eye.x, 0.0, at.z - eye.z).normalize_or_zero();
    let ahead = if ahead == Vec3::ZERO {
        Vec3::NEG_Z
    } else {
        ahead
    };
    let half = ahead.cross(Vec3::Y) * (size.x / 2.0);
    let top = Vec3::Y * size.y;
    [
        at + top / 2.0,
        at - half,
        at + half,
        at - half + top,
        at + half + top,
    ]
}

/// `at` moved toward `eye` along the line between them, so that every probe
/// a hit lies on or in front of ends up a margin short of it. `hits` are
/// distances from `eye` along the way to each probe.
fn pulled(eye: Vec3, at: Vec3, probes: &[Vec3; PROBES], hits: &[Option<f32>; PROBES]) -> Vec3 {
    let mut ratio = 1.0_f32;
    for (probe, hit) in probes.iter().zip(hits) {
        let Some(hit) = hit else {
            continue;
        };
        let reach = eye.distance(*probe);
        // A hit at the probe is a pane flush with or already into the surface.
        // Asking only for hits well in front leaves a sliver of the pane
        // buried, so pull for anything short of a margin clear of it.
        let wanted = hit - MARGIN;
        if wanted < reach {
            ratio = ratio.min(wanted / reach);
        }
    }
    if ratio >= 1.0 {
        return at;
    }
    let offset = at - eye;
    let floor = (NEAREST / offset.length()).min(1.0);
    snap(eye + offset * ratio.max(floor))
}

fn snap(at: Vec3) -> Vec3 {
    let round = |metres: f32| (metres / STEP).round() * STEP;
    Vec3::new(round(at.x), round(at.y), round(at.z))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EYE: Vec3 = Vec3::ZERO;
    const AT: Vec3 = Vec3::new(0.0, 0.0, -4.0);
    const SIZE: Vec2 = Vec2::new(1.0, 0.4);

    fn pull(hits: [Option<f32>; PROBES]) -> Vec3 {
        pulled(EYE, AT, &probes(EYE, AT, SIZE), &hits)
    }

    #[test]
    fn the_probes_are_the_middle_and_corners_across_the_view() {
        let probes = probes(EYE, AT, SIZE);
        assert!(probes[0].abs_diff_eq(Vec3::new(0.0, 0.2, -4.0), 1.0e-5));
        for corner in [
            Vec3::new(-0.5, 0.0, -4.0),
            Vec3::new(0.5, 0.0, -4.0),
            Vec3::new(-0.5, 0.4, -4.0),
            Vec3::new(0.5, 0.4, -4.0),
        ] {
            assert!(
                probes.iter().any(|p| p.abs_diff_eq(corner, 1.0e-5)),
                "missing {corner:?}"
            );
        }
    }

    #[test]
    fn a_clear_view_leaves_the_pane_where_it_was() {
        assert_eq!(pull([None; PROBES]), AT);
        assert_eq!(pull([Some(4.5); PROBES]), AT);
    }

    #[test]
    fn a_hit_in_front_pulls_the_pane_to_a_margin_short_of_it() {
        let at = pull([Some(2.0), None, None, None, None]);
        assert!((at.z + 1.98).abs() < 0.01, "{at:?}");
    }

    #[test]
    fn the_nearest_hit_decides() {
        let at = pull([Some(3.0), Some(1.0), None, Some(2.0), None]);
        assert!((at.z + 0.97).abs() < 0.01, "{at:?}");
    }

    #[test]
    fn a_pane_flush_with_a_surface_steps_a_margin_clear_of_it() {
        let at = pull([Some(4.0), None, None, None, None]);
        assert!(at.z > -4.0 && at.z < -3.9, "{at:?}");
        assert!(
            (-at.z - (4.0 - MARGIN)).abs() < STEP + 1.0e-4,
            "a flush hit is pulled a margin clear, got {at:?}"
        );
    }

    #[test]
    fn a_pane_is_never_pulled_into_the_eye() {
        let at = pull([Some(0.1); PROBES]);
        assert!((at.z + NEAREST).abs() < STEP, "{at:?}");
    }
}
