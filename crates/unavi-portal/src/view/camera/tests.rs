use bevy::camera::visibility::RenderLayers;
use bevy_vrm::first_person::FIRST_PERSON_LAYER;
use unavi_avatar::render_layers::{
    AVATAR_RENDER_LAYER,
    avatar_first_person_layers,
};

use super::{
    still_layers,
    third_person_layers,
};
use crate::render_layers::PORTAL_RENDER_LAYER;

/// The layers the local agent's camera renders.
fn main_camera() -> RenderLayers {
    RenderLayers::from_layers(&[
        0,
        PORTAL_RENDER_LAYER,
        AVATAR_RENDER_LAYER,
        FIRST_PERSON_LAYER,
    ])
}

/// Every layer an avatar mesh can end up on.
fn person_layers() -> [RenderLayers; 3] {
    let layers = avatar_first_person_layers();
    [
        layers.both,
        layers.first_person_only,
        layers.third_person_only,
    ]
}

#[test]
fn a_still_leaves_out_every_avatar_mesh() {
    let still = still_layers(&main_camera());
    for layers in person_layers() {
        assert!(
            !still.intersects(&layers),
            "{layers:?} avatar mesh is in the still"
        );
    }
}

#[test]
fn a_still_keeps_the_world() {
    assert!(still_layers(&main_camera()).intersects(&RenderLayers::layer(0)));
}

#[test]
fn a_still_has_no_portal_planes() {
    assert!(!still_layers(&main_camera()).intersects(&RenderLayers::layer(PORTAL_RENDER_LAYER)));
}

#[test]
fn the_main_camera_still_sees_the_avatars_it_saw() {
    let main = main_camera();
    let layers = avatar_first_person_layers();
    for layers in [layers.both, layers.first_person_only] {
        assert!(main.intersects(&layers), "{layers:?} vanished");
    }
}

#[test]
fn live_views_still_see_the_avatars_they_saw() {
    let live = third_person_layers(&main_camera());
    let layers = avatar_first_person_layers();
    for layers in [layers.both, layers.third_person_only] {
        assert!(live.intersects(&layers), "{layers:?} vanished");
    }
}
