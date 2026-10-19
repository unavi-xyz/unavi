use bevy::{
    prelude::*,
    transform::TransformPlugin,
};

use super::{
    PortalVisual,
    apply_visual,
    choose_visual,
};
use crate::{
    destination::Destination,
    portal::{
        Portal,
        PortalState,
    },
    view::material::PortalMaterial,
};

fn setup() -> App {
    let mut app = App::new();
    app.add_plugins((
        bevy::app::TaskPoolPlugin::default(),
        bevy::asset::AssetPlugin::default(),
        TransformPlugin,
    ))
    .init_asset::<Image>()
    .init_asset::<PortalMaterial>()
    .init_resource::<Time>()
    .add_systems(Update, (choose_visual, apply_visual).chain());
    app
}

#[test]
fn an_open_portal_is_drawn_with_the_portal_material_and_a_closed_one_is_hidden() {
    let mut app = setup();
    let portal = app.world_mut().spawn((Portal, PortalState::Closed)).id();
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(portal),
        Some(&Visibility::Hidden)
    );

    app.world_mut()
        .entity_mut(portal)
        .insert(PortalState::Loading);
    app.update();
    assert_eq!(
        app.world().get::<Visibility>(portal),
        Some(&Visibility::Inherited)
    );
    assert!(
        app.world()
            .get::<MeshMaterial3d<PortalMaterial>>(portal)
            .is_some(),
        "a loading portal draws the rim and fill, not a fallback material"
    );
}

#[test]
fn a_one_way_portal_shows_the_loading_fill_until_its_still_is_taken() {
    let mut app = setup();
    let root = app.world_mut().spawn_empty().id();
    let portal = app
        .world_mut()
        .spawn((Portal, PortalState::Open, Destination(root)))
        .id();
    app.update();
    assert_eq!(
        app.world().get::<PortalVisual>(portal),
        Some(&PortalVisual::Loading)
    );
}

#[test]
fn a_linked_portal_outside_the_view_budget_shows_the_veil() {
    let mut app = setup();
    let far = app.world_mut().spawn(Portal).id();
    let near = app
        .world_mut()
        .spawn((Portal, PortalState::Open, Destination(far)))
        .id();
    app.update();
    assert_eq!(
        app.world().get::<PortalVisual>(near),
        Some(&PortalVisual::Veil)
    );
}
