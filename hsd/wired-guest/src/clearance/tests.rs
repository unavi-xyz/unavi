use super::*;

const EYE: Vec3 = Vec3::ZERO;
const AT: Vec3 = Vec3::new(0.0, 0.0, -4.0);
const SIZE: Vec2 = Vec2::new(1.0, 0.4);

fn pull(hits: [Option<f32>; PROBES]) -> Vec3 {
    pulled(EYE, AT, &probes(EYE, AT, SIZE), &hits)
}

/// A cast against a solid floor whose top is at `height`.
fn floor(height: f32) -> impl FnMut(Vec3, Vec3) -> Option<f32> {
    move |from: Vec3, to: Vec3| {
        let (above, below) = (from.y - height, to.y - height);
        if above < 0.0 {
            return Some(0.0);
        }
        (below <= 0.0).then(|| from.distance(to) * above / (above - below))
    }
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

#[test]
fn a_pane_sunk_into_the_floor_rises_a_margin_clear_of_it() {
    let eye = Vec3::new(0.0, 1.6, 0.0);
    let foot = Vec3::new(0.0, -0.3, -1.2);
    let at = clear(eye, foot, SIZE, floor(0.0));
    assert!(
        (at.y - MARGIN).abs() < STEP + 1.0e-4,
        "the bottom edge stands a margin over the floor, got {at:?}"
    );
    assert!(
        (at.z - foot.z).abs() < 0.05,
        "rising does not pull it in, got {at:?}"
    );
}

#[test]
fn a_pane_above_the_floor_is_left_where_it_was() {
    let eye = Vec3::new(0.0, 1.6, 0.0);
    let foot = Vec3::new(0.0, 0.8, -1.2);
    assert_eq!(clear(eye, foot, SIZE, floor(0.0)), foot);
}
