use smol_str::SmolStr;
use wired_guest::math::{
    Quat,
    Transform,
    Vec2,
};

use super::*;
use crate::{
    component::placard::Content,
    machinery::layout::Layout,
    part::{
        spec::{
            Arrange,
            Role,
        },
        view::{
            Aim,
            Body,
        },
    },
    tokens::state::Condition,
};

const PITCH: Vec2 = Vec2::new(0.11, 0.11);

fn mote(role: Role, label: &str, description: Option<&str>) -> MoteSpec {
    MoteSpec {
        role,
        label: SmolStr::new(label),
        description: description.map(SmolStr::new),
        condition: Condition::default(),
        icon: false,
        glyph: None,
        tint: None,
        film: crate::tokens::palette::FILM,
        frost: 0.0,
        notice: None,
    }
}

fn root_specs() -> [MoteSpec; 5] {
    let group = Role::Group {
        children: 3,
        groups:   0,
        arrange:  Arrange::Bed,
    };
    [
        mote(group, "Hand", Some("What is in your hand.")),
        mote(Role::Cast, "Home", Some("Go home.")),
        mote(group, "Places", None),
        mote(group, "Pocket", None),
        mote(group, "Self", None),
    ]
}

fn frame(aim: Option<Aim>) -> Frame {
    Frame {
        anchor: Transform {
            translation: Vec3::ZERO,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        },
        aim,
        hand: None,
        delta: 0.016,
    }
}

fn aim_at(local: Vec2) -> Aim {
    Aim {
        local,
        world: Vec3::new(local.x, local.y, 0.0),
    }
}

fn bed() -> Surface {
    Surface::new(9, Tuning::DEFAULT, Palette::DEFAULT)
}

/// Steps a bed's root level `frames` times with the pointer at `aim`.
fn step(surface: &mut Surface, specs: &[MoteSpec], aim: Option<Vec2>, frames: usize) {
    for _ in 0..frames {
        surface.update(specs, Layout::bed(PITCH, true), 0, &frame(aim.map(aim_at)));
    }
}

fn texts(surface: &Surface) -> Vec<String> {
    surface.placard().map_or_else(Vec::new, |view| {
        view.sheet
            .runs
            .text
            .iter()
            .map(|run| run.text.to_string())
            .collect()
    })
}

#[test]
fn the_middle_of_a_bed_is_the_orb_and_every_other_place_is_a_key() {
    let mut surface = bed();
    step(&mut surface, &root_specs(), None, 2);
    let bodies = surface
        .views()
        .iter()
        .map(|view| view.body)
        .collect::<Vec<_>>();
    assert_eq!(
        bodies,
        [Body::Orb, Body::Key, Body::Key, Body::Key, Body::Key]
    );
}

#[test]
fn keys_are_one_size_whatever_they_hold() {
    let mut surface = bed();
    step(&mut surface, &root_specs(), None, 2);
    let views = surface.views();
    assert!(
        (views[1].radius - views[2].radius).abs() < 1.0e-6,
        "a cast and a group are the same key"
    );
    assert!(views[1].pips.count == 0, "a key previews nothing");
}

#[test]
fn a_level_inside_a_bed_holds_the_way_back_in_the_orb() {
    let mut surface = bed();
    let mut specs = vec![mote(Role::Parent { depth: 1 }, "Tools", None)];
    specs.extend((0..8).map(|index| mote(Role::Action, &format!("Tool {index}"), None)));
    for _ in 0..2 {
        surface.update(&specs, Layout::bed(PITCH, false), 1, &frame(None));
    }
    let views = surface.views();
    assert_eq!(views.len(), 9);
    assert_eq!(views[0].body, Body::Orb);
    assert!(views[1..].iter().all(|view| view.body == Body::Key));
}

#[test]
fn a_bed_stands_on_a_slab_the_size_of_its_cells() {
    let mut surface = bed();
    step(&mut surface, &root_specs(), None, 2);
    let slab = surface.bed().expect("a slab");
    assert!((slab.extents.x - PITCH.x * 1.5).abs() < 1.0e-6);
    assert!((slab.extents.y - PITCH.y * 1.5).abs() < 1.0e-6);

    let mut ring = bed();
    ring.update(&root_specs(), Layout::centred(4, 0.155), 1, &frame(None));
    assert!(ring.bed().is_none(), "a ring has no slab");
}

#[test]
fn the_page_previews_the_attended_key_without_waiting_for_a_dwell() {
    let mut surface = bed();
    step(
        &mut surface,
        &root_specs(),
        Some(Vec2::new(0.0, PITCH.y)),
        2,
    );
    assert_eq!(texts(&surface), ["Home", "Go home."]);
    let view = surface.placard().expect("a page");
    let top = Layout::bed(PITCH, true).extents(&Tuning::DEFAULT).y;
    assert!(view.position.y > top, "above the bed, never across it");
    assert!(view.position.x.abs() < 1.0e-6, "centred on it");
}

#[test]
fn a_page_with_nothing_to_show_is_no_page() {
    let mut surface = bed();
    step(&mut surface, &root_specs(), None, 2);
    assert!(surface.placard().is_none());
}

#[test]
fn the_root_rests_on_what_the_consumer_set() {
    let mut surface = bed();
    surface.set_resting(Some(Content::Resting {
        title: SmolStr::new_static("Plaza"),
        lines: vec![SmolStr::new_static("14:07")],
    }));
    step(&mut surface, &root_specs(), None, 2);
    assert_eq!(texts(&surface), ["Plaza", "14:07"]);

    step(
        &mut surface,
        &root_specs(),
        Some(Vec2::new(PITCH.x, 0.0)),
        2,
    );
    assert_eq!(
        texts(&surface),
        ["Places"],
        "a key attended is the page, not the resting one"
    );
}

#[test]
fn a_level_inside_the_bed_does_not_borrow_the_roots_resting_page() {
    let mut surface = bed();
    surface.set_resting(Some(Content::Resting {
        title: SmolStr::new_static("Plaza"),
        lines: Vec::new(),
    }));
    let specs = [mote(Role::Parent { depth: 1 }, "Tools", None)];
    for _ in 0..2 {
        surface.update(&specs, Layout::bed(PITCH, false), 1, &frame(None));
    }
    assert!(surface.placard().is_none());
}

#[test]
fn a_pressed_key_keeps_its_page_after_the_pointer_leaves() {
    let mut surface = bed();
    let specs = root_specs();
    step(&mut surface, &specs, Some(Vec2::new(0.0, PITCH.y)), 2);
    surface.keep(&specs[1]);
    step(&mut surface, &specs, None, 2);
    assert_eq!(texts(&surface), ["Home", "Go home."]);
}

#[test]
fn a_kept_page_gives_way_to_the_key_attended_and_ends_with_the_level() {
    let mut surface = bed();
    let specs = root_specs();
    surface.keep(&specs[1]);
    step(&mut surface, &specs, Some(Vec2::new(PITCH.x, 0.0)), 2);
    assert_eq!(texts(&surface), ["Places"]);

    step(&mut surface, &specs, None, 2);
    assert_eq!(texts(&surface), ["Home", "Go home."]);

    surface.relevel();
    step(&mut surface, &specs, None, 2);
    assert!(surface.placard().is_none(), "opening a level lets it go");
}

#[test]
fn a_dismissed_bed_has_no_page() {
    let mut surface = bed();
    surface.set_resting(Some(Content::Resting {
        title: SmolStr::new_static("Plaza"),
        lines: Vec::new(),
    }));
    surface.set_open(false);
    step(&mut surface, &root_specs(), None, 2);
    assert!(surface.placard().is_none());
}
