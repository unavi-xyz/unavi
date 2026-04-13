use std::time::Duration;

use setup::logs::count_logs_with;
use unavi_script::permissions::ScriptPermissions;

mod setup;

#[test]
fn script_invalid() {
    let mut app =
        setup::setup_test_app("invalid", Some(vec![0; 128]), ScriptPermissions::default());

    // Wait until the error log appears or timeout.
    let ready = setup::wait_until(
        &mut app,
        || count_logs_with("error instantiating script component") >= 1,
        Duration::from_secs(20),
    );

    assert!(ready, "expected error log within timeout");
    assert_eq!(count_logs_with("error instantiating script component"), 1);
}
