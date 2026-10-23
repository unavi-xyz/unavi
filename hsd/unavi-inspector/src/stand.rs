//! Where the slate stands for what it describes: on the line from the eye to
//! it, no farther than reading distance, a little to the viewer's right so it
//! does not cover it, and clear of the floor and of anything in front.

use wired_guest::{
    clearance,
    math::{
        Ray,
        Vec2,
        Vec3,
    },
};

use crate::wired::{
    core::ids::DocumentId,
    physics::simulation::{
        RayFilter,
        raycast,
    },
};

/// Metres from the eye a slate stands at most, and at least.
const READING: f32 = 0.9;
const NEAREST: f32 = 0.55;
/// Metres short of what it describes a slate stands, when that is nearer than
/// reading distance.
const SHORT: f32 = 0.3;
/// Metres the pane's near edge stands right of the line to what it describes.
const ASIDE: f32 = 0.08;
/// Metres the pane's top stands above the eye, so its rows sit just below the
/// line of sight.
const RISE: f32 = 0.06;

/// The top centre of a `size` slate describing `subject`, as seen from `eye`.
/// Colliders of `own`, the slate's document, do not count.
pub fn stand(eye: Vec3, subject: Vec3, size: Vec2, own: DocumentId) -> Vec3 {
    let foot = wanted(eye, subject, size);
    let filter = RayFilter {
        exclude_local_agent: true,
        exclude_documents:   vec![own],
    };
    let foot = clearance::clear(eye, foot, size, |from, to| cast(from, to, &filter));
    foot + Vec3::Y * size.y
}

/// The bottom centre a slate asks for before anything is in its way.
fn wanted(eye: Vec3, subject: Vec3, size: Vec2) -> Vec3 {
    let flat = Vec3::new(subject.x - eye.x, 0.0, subject.z - eye.z);
    let distance = flat.length();
    let ahead = if distance < f32::EPSILON {
        Vec3::NEG_Z
    } else {
        flat / distance
    };
    let along = (distance - SHORT).clamp(NEAREST, READING);
    let right = ahead.cross(Vec3::Y);
    let top = eye.y + RISE;
    Vec3::new(eye.x, top - size.y, eye.z) + ahead * along + right * (ASIDE + size.x / 2.0)
}

/// The distance from `from` to the first collider on the way to `to`.
fn cast(from: Vec3, to: Vec3, filter: &RayFilter) -> Option<f32> {
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
            eprintln!("inspector: clearance raycast: {err:?}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: Vec2 = Vec2::new(0.46, 0.6);
    const EYE: Vec3 = Vec3::new(0.0, 1.6, 0.0);

    #[test]
    fn a_far_subject_gets_its_slate_at_reading_distance() {
        let foot = wanted(EYE, Vec3::new(0.0, 0.0, -10.0), SIZE);
        assert!((foot.z + READING).abs() < 1.0e-5, "{foot:?}");
        assert!(
            (foot.y + SIZE.y - (EYE.y + RISE)).abs() < 1.0e-5,
            "{foot:?}"
        );
    }

    #[test]
    fn the_slate_stands_to_the_right_of_the_line_to_its_subject() {
        let foot = wanted(EYE, Vec3::new(0.0, 0.0, -10.0), SIZE);
        assert!(
            (foot.x - SIZE.x / 2.0 - ASIDE).abs() < 1.0e-5,
            "near edge an aside off the line, got {foot:?}"
        );
    }

    #[test]
    fn a_near_subject_gets_its_slate_short_of_it_but_not_in_the_face() {
        let foot = wanted(EYE, Vec3::new(0.0, 0.5, -1.0), SIZE);
        assert!((foot.z + 0.7).abs() < 1.0e-5, "{foot:?}");
        let foot = wanted(EYE, Vec3::new(0.0, 0.5, -0.2), SIZE);
        assert!((foot.z + NEAREST).abs() < 1.0e-5, "{foot:?}");
    }
}
