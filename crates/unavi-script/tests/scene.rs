use std::time::Duration;

use setup::logs::{has_error_log, has_log};

mod setup;

#[test]
fn scene_api() {
    let mut app = setup::setup_test_app("scene", None);
    let ready = setup::wait_until(
        &mut app,
        || has_log("tests complete"),
        Duration::from_secs(20),
    );
    assert!(ready, "scene tests did not complete within timeout");
    assert!(!has_error_log(), "scene tests logged errors");
}
