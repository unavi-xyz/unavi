use std::time::Duration;

use setup::logs::{has_error_log, has_log};
use unavi_script::permissions::ApiPermissions;

mod setup;

#[test]
fn scene_api() {
    let mut app = setup::setup_test_app("scene", ApiPermissions::default());
    let ready = setup::wait_until(
        &mut app,
        || has_log("tests complete"),
        Duration::from_secs(20),
    );
    assert!(ready, "scene tests did not complete within timeout");
    assert!(!has_error_log(), "scene tests logged errors");
}

#[test]
fn scene_create_document() {
    let mut app = setup::setup_test_app("scene", ApiPermissions::system());
    let ready = setup::wait_until(
        &mut app,
        || has_log("tests complete"),
        Duration::from_secs(30),
    );
    assert!(ready, "scene tests timed out");
    assert!(!has_error_log(), "scene tests logged errors");

    // Flush commands from the script.
    setup::tick_app(&mut app);
    setup::tick_app(&mut app);

    let node_count = app
        .world_mut()
        .query::<&bevy_hsd::NodeId>()
        .iter(app.world())
        .count();
    assert!(node_count > 0, "no NodeId entities spawned");

    let doc_count = app
        .world_mut()
        .query::<&bevy_hsd::HsdRecordId>()
        .iter(app.world())
        .count();
    assert!(
        doc_count >= 2,
        "expected at least 2 HsdRecordId entities (own + created)"
    );
}
