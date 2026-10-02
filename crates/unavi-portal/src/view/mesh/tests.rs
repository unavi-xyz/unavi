use bevy::{
    prelude::*,
    transform::TransformPlugin,
};

use super::apply_active_material;
use crate::{
    portal::{
        Portal,
        PortalState,
    },
    view::material::PortalFallbacks,
};

fn setup() -> App {
    let mut app = App::new();
    app.add_plugins((
        bevy::app::TaskPoolPlugin::default(),
        bevy::asset::AssetPlugin::default(),
        TransformPlugin,
    ))
    .init_asset::<StandardMaterial>()
    .init_resource::<PortalFallbacks>()
    .add_systems(Update, apply_active_material);
    app
}

fn spawn_portal(app: &mut App, state: PortalState) -> Entity {
    app.world_mut().spawn((Portal, state)).id()
}

#[test]
fn opening_a_portal_makes_it_visible() {
    let mut app = setup();
    let portal = spawn_portal(&mut app, PortalState::Closed);
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
        Some(&Visibility::Visible)
    );
}
