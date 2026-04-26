use std::time::Duration;

use setup::logs::{has_error_log, has_log};
use unavi_script::permissions::ApiPermissions;

mod setup;

#[test]
fn event_api() {
    let mut app = setup::setup_test_app("event", ApiPermissions::default());
    let ready = setup::wait_until(
        &mut app,
        || has_log("tests complete"),
        Duration::from_secs(20),
    );
    assert!(ready, "event tests did not complete within timeout");
    assert!(!has_error_log(), "event tests logged errors");
}
