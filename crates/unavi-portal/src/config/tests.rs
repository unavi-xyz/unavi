use bevy::prelude::*;
use bevy_hsd::attributes::portal::PortalConfig;
use hsd::attributes::portal::PortalAttr;

use super::{
    PortalRefused,
    admit_refused_portals,
    sync_portal_config,
};
use crate::portal::{
    Portal,
    PortalLimits,
};

fn setup(max_portals: usize) -> App {
    let mut app = App::new();
    app.insert_resource(PortalLimits { max_portals })
        .add_observer(sync_portal_config)
        .add_systems(Update, admit_refused_portals);
    app
}

fn spawn_config(app: &mut App) -> Entity {
    app.world_mut()
        .spawn(PortalConfig(PortalAttr {
            size_x: 1.0,
            size_y: 1.0,
            ..default()
        }))
        .id()
}

#[test]
fn portals_within_the_cap_all_open() {
    let mut app = setup(2);
    let a = spawn_config(&mut app);
    let b = spawn_config(&mut app);

    assert!(app.world().get::<Portal>(a).is_some());
    assert!(app.world().get::<Portal>(b).is_some());
}

#[test]
fn a_portal_past_the_cap_is_refused() {
    let mut app = setup(1);
    let a = spawn_config(&mut app);
    let b = spawn_config(&mut app);

    assert!(app.world().get::<Portal>(a).is_some());
    assert!(
        app.world().get::<Portal>(b).is_none(),
        "portal past the cap should not open"
    );
    assert!(
        app.world().get::<PortalRefused>(b).is_some(),
        "refused prim should be marked so it can be retried later"
    );
}

#[test]
fn a_refused_portal_opens_once_capacity_frees() {
    let mut app = setup(1);
    let a = spawn_config(&mut app);
    let b = spawn_config(&mut app);
    assert!(app.world().get::<Portal>(a).is_some());
    assert!(app.world().get::<PortalRefused>(b).is_some());

    app.world_mut().entity_mut(a).despawn();
    app.update();

    assert!(
        app.world().get::<Portal>(b).is_some(),
        "refused portal should open once a slot frees"
    );
    assert!(app.world().get::<PortalRefused>(b).is_none());
}

#[test]
fn a_config_change_on_an_already_open_portal_is_not_refused() {
    let mut app = setup(1);
    let a = spawn_config(&mut app);
    assert!(app.world().get::<Portal>(a).is_some());

    // Re-inserting the config (as a real size/destination change would)
    // must not count against the cap a second time.
    app.world_mut()
        .entity_mut(a)
        .insert(PortalConfig(PortalAttr {
            size_x: 2.0,
            size_y: 2.0,
            ..default()
        }));

    assert!(app.world().get::<Portal>(a).is_some());
}
