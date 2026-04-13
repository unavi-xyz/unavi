use std::time::Duration;

use setup::{
    logs::{LOGS, has_error_log, has_log},
    tick_app,
};
use unavi_script::permissions::ScriptPermissions;

mod setup;

#[test]
fn script_stall() {
    let mut app = setup::setup_test_app("stall", None, ScriptPermissions::default());

    let ready = setup::wait_until(
        &mut app,
        || has_log("hello from init"),
        Duration::from_secs(20),
    );
    assert!(ready, "script did not log init message within timeout");

    assert_eq!(
        LOGS.logs
            .lock()
            .expect("test value expected")
            .iter()
            .filter(|line| line.contains("hello from init"))
            .count(),
        1
    );

    // Execute script tick — should never complete, but ECS should continue.
    for _ in 0..5 {
        tick_app(&mut app);
    }
    assert!(
        !LOGS
            .logs
            .lock()
            .expect("test value expected")
            .iter()
            .any(|line| line.contains("hello from tick"))
    );

    assert!(!has_error_log());
}
