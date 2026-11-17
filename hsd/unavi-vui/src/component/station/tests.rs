use std::collections::{
    HashMap,
    HashSet,
};

use wired_guest::math::{
    Transform,
    Vec3,
};

use super::{
    Station,
    dock::Screen,
    stage::Stage,
    tiling::{
        Axis,
        Cells,
        Node,
        Part,
        Tiling,
    },
};
use crate::{
    tokens::tuning::Tuning,
    wired::input::types::PointerKind,
};

/// Panes a station arranges, recording what it did to them.
#[derive(Default)]
struct Fake {
    sizes:    HashMap<u32, (u32, u32)>,
    hung:     HashMap<u32, Transform>,
    shown:    HashMap<u32, bool>,
    pulls:    HashMap<u32, PointerKind>,
    carried:  HashSet<u32>,
    at:       HashMap<u32, Vec3>,
    unmapped: HashSet<u32>,
}

impl Stage<u32> for Fake {
    fn cells(&self, id: u32) -> (u32, u32) {
        self.sizes.get(&id).copied().unwrap_or((40, 30))
    }

    fn hang(&mut self, id: u32, pose: Transform) {
        self.hung.insert(id, pose);
    }

    fn show(&mut self, id: u32, shown: bool) {
        self.shown.insert(id, shown);
    }

    fn take_pulled(&mut self, id: u32) -> Option<PointerKind> {
        self.pulls.remove(&id)
    }

    fn carry(&mut self, id: u32, at: Vec3, _: PointerKind) {
        self.carried.insert(id);
        self.at.insert(id, at);
    }

    fn carried(&self, id: u32) -> bool {
        self.carried.contains(&id)
    }

    fn position(&self, id: u32) -> Option<Vec3> {
        self.at.get(&id).copied()
    }

    fn unmount(&mut self, id: u32) {
        self.unmapped.insert(id);
    }
}

fn eye() -> Transform {
    Transform::from_translation(Vec3::new(2.0, 1.6, 3.0))
}

fn leaf(id: u32) -> Part<u32> {
    Part {
        weight: 1,
        node:   Node::Tabs(vec![id]),
    }
}

/// Three panes side by side.
fn row() -> Station<u32> {
    let mut station = Station::new(
        Tiling::new(Node::Split {
            axis:  Axis::Across,
            parts: vec![leaf(1), leaf(2), leaf(3)],
        })
        .unwrap(),
    );
    station.summon();
    station
}

fn step(station: &mut Station<u32>, stage: &mut Fake) {
    station.step(stage, &eye(), &Tuning::DEFAULT, None);
}

fn grip() -> PointerKind {
    PointerKind::RightHand
}

#[test]
fn summoning_stands_every_pane_around_the_viewer() {
    let mut station = row();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);

    let eye = eye().translation;
    let reach = Tuning::DEFAULT.station_reach;
    for id in 1..=3 {
        assert!(stage.shown[&id]);
        let away = stage.hung[&id].translation - eye;
        let flat = away.x.hypot(away.z);
        assert!((flat - reach).abs() < 1.0e-3, "pane {id} is {flat} from the axis");
    }
    assert!(stage.hung[&1].translation.x < stage.hung[&3].translation.x, "left to right");
    let middle = stage.hung[&2].translation;
    assert!(
        (middle.x - eye.x).abs() < 1.0e-3 && (middle.z - (eye.z - reach)).abs() < 1.0e-3,
        "the middle pane is straight ahead"
    );
}

#[test]
fn a_dismissed_station_hides_its_panes_and_a_summoned_one_is_planted_afresh() {
    let mut station = row();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);
    let first = stage.hung[&2];

    station.dismiss();
    step(&mut station, &mut stage);
    assert!((1..=3).all(|id| !stage.shown[&id]));

    station.summon();
    station.step(
        &mut stage,
        &Transform::from_translation(Vec3::new(-5.0, 1.6, 0.0)),
        &Tuning::DEFAULT,
        None,
    );
    assert!((stage.hung[&2].translation.x - first.translation.x).abs() > 1.0);
    assert!(stage.shown[&2]);
}

#[test]
fn a_pane_pulled_out_floats_where_it_hung_and_leaves_its_place_empty() {
    let mut station = row();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);
    let others = (stage.hung[&1], stage.hung[&3]);
    let hung = stage.hung[&2];

    stage.pulls.insert(2, grip());
    step(&mut station, &mut stage);

    assert!(stage.carried.contains(&2));
    assert_eq!(stage.at[&2], hung.translation);
    assert_eq!((stage.hung[&1], stage.hung[&3]), others, "the rest do not move up");
}

#[test]
fn a_pane_let_go_near_its_place_goes_back_to_it() {
    let mut station = row();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);
    let place = stage.hung[&2].translation;
    stage.pulls.insert(2, grip());
    step(&mut station, &mut stage);

    stage.carried.remove(&2);
    stage.at.insert(2, place + Vec3::new(0.1, 0.1, 0.1));
    step(&mut station, &mut stage);

    assert!(station.afloat.is_empty());
    assert_eq!(station.tiling.items(), [1, 2, 3]);
    assert_eq!(stage.hung[&2].translation, place);
}

#[test]
fn a_pane_let_go_beside_another_goes_to_the_nearest_place() {
    let mut station = row();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);
    let beside = stage.hung[&3].translation;
    stage.pulls.insert(2, grip());
    step(&mut station, &mut stage);

    stage.carried.remove(&2);
    stage.at.insert(2, beside + Vec3::new(0.0, 0.05, 0.1));
    step(&mut station, &mut stage);

    assert!(station.afloat.is_empty());
    let leaves = station.tiling.leaves(Default::default());
    assert_eq!(leaves.len(), 2, "the place it left collapsed");
    assert_eq!(leaves[1].tabs, [2, 3], "it came first, so it is the one shown");
    assert!(stage.shown[&2]);
    assert!(!stage.shown[&3]);
}

#[test]
fn a_pane_let_go_far_from_the_station_stays_afloat() {
    let mut station = row();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);
    stage.pulls.insert(2, grip());
    step(&mut station, &mut stage);

    stage.carried.remove(&2);
    stage.at.insert(2, Vec3::new(2.0, 1.0, 3.0));
    step(&mut station, &mut stage);
    step(&mut station, &mut stage);

    assert_eq!(station.afloat.len(), 1);
    assert!(stage.shown[&2], "still up, wherever it floats");
}

#[test]
fn a_pane_afloat_can_be_picked_up_and_put_back() {
    let mut station = row();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);
    let place = stage.hung[&1].translation;
    stage.pulls.insert(1, grip());
    step(&mut station, &mut stage);
    stage.carried.remove(&1);
    stage.at.insert(1, Vec3::new(2.0, 1.0, 3.0));
    step(&mut station, &mut stage);

    stage.carried.insert(1);
    step(&mut station, &mut stage);
    assert_eq!(station.afloat.len(), 1, "carried again");
    stage.carried.remove(&1);
    stage.at.insert(1, place);
    step(&mut station, &mut stage);

    assert!(station.afloat.is_empty());
}

#[test]
fn pulling_the_pane_shown_in_a_tabbed_leaf_shows_the_next() {
    let mut station = Station::new(
        Tiling::new(Node::Tabs(vec![1, 2])).unwrap(),
    );
    station.summon();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);
    assert!(stage.shown[&1] && !stage.shown[&2]);

    stage.pulls.insert(1, grip());
    step(&mut station, &mut stage);
    step(&mut station, &mut stage);

    assert!(stage.shown[&1], "afloat is still up");
    assert!(stage.shown[&2], "and the tab behind it takes the place");
}

#[test]
fn a_freed_pane_leaves_the_station() {
    let mut station = row();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);

    station.forget(&2);
    assert_eq!(station.tiling.items(), [1, 3]);
    step(&mut station, &mut stage);
    assert!(stage.hung[&1].translation.x < stage.hung[&3].translation.x);
}

#[test]
fn clearing_a_station_takes_its_panes_off_their_mounts_but_not_those_afloat() {
    let mut station = row();
    let mut stage = Fake::default();
    step(&mut station, &mut stage);
    stage.pulls.insert(2, grip());
    step(&mut station, &mut stage);

    station.clear(&mut stage);

    assert_eq!(stage.unmapped, HashSet::from([1, 3]));
}

#[test]
fn docking_lays_the_panes_against_the_screens_edges_and_they_follow_the_eye() {
    let screen = Screen {
        pixels: Cells {
            w: 2560,
            h: 1440,
        },
        fov:    1.0,
    };
    let mut station = row();
    let mut stage = Fake::default();
    station.dock();
    station.step(&mut stage, &eye(), &Tuning::DEFAULT, Some(screen));

    let eye = eye().translation;
    for id in 1..=3 {
        assert!(stage.shown[&id]);
        let at = stage.hung[&id].translation;
        assert!(at.z < eye.z, "pane {id} is in front of the eye");
        assert!((at.y - eye.y).abs() < 0.2, "pane {id} is level with the eye");
    }
    let (left, right) = (
        stage.hung[&1].translation.x,
        stage.hung[&3].translation.x,
    );
    assert!(left < eye.x && right > eye.x, "one either side of the middle");

    // The station follows the eye rather than staying where it was summoned.
    let elsewhere = Transform::from_translation(Vec3::new(-4.0, 1.6, -2.0));
    station.step(&mut stage, &elsewhere, &Tuning::DEFAULT, Some(screen));
    let at = stage.hung[&2].translation;
    assert!((at.x - elsewhere.translation.x).abs() < 0.2);
    assert!((at.z - (elsewhere.translation.z - Tuning::DEFAULT.pane_reading)).abs() < 0.3);
}

#[test]
fn a_docked_station_stands_on_the_arc_in_a_headset() {
    let mut station = row();
    let mut stage = Fake::default();
    station.dock();
    station.step(&mut stage, &eye(), &Tuning::DEFAULT, None);
    let away = stage.hung[&1].translation - eye().translation;
    let reach = Tuning::DEFAULT.station_reach;
    assert!((away.x.hypot(away.z) - reach).abs() < 1.0e-3);
}
