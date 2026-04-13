use std::time::Duration;

use setup::logs::{has_error_log, has_log};
use unavi_script::permissions::ScriptPermissions;

mod setup;

#[test]
fn input_api() {
    let mut app = setup::setup_test_app("input", None, ScriptPermissions::default());
    let ready = setup::wait_until(
        &mut app,
        || has_log("tests complete"),
        Duration::from_secs(20),
    );
    assert!(ready, "input tests did not complete within timeout");
    assert!(!has_error_log(), "input tests logged errors");
}
