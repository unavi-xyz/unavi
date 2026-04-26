use std::time::Duration;

use setup::{
    logs::{LOGS, has_error_log, has_log},
    tick_app,
};
use unavi_script::permissions::ApiPermissions;

mod setup;

#[test]
fn script_log() {
    let mut app = setup::setup_test_app("log", ApiPermissions::default());

    let ready = setup::wait_until(
        &mut app,
        || has_log("hello from init"),
        Duration::from_secs(20),
    );
    assert!(ready, "script did not log init message within timeout");

    for _ in 0..5 {
        tick_app(&mut app);
    }

    let n_inits = LOGS
        .logs
        .lock()
        .expect("test value expected")
        .iter()
        .filter(|line| line.contains("hello from init"))
        .count();
    assert_eq!(n_inits, 1, "has 1 startup log");

    let n_ticks = LOGS
        .logs
        .lock()
        .expect("test value expected")
        .iter()
        .filter(|line| line.contains("hello from tick"))
        .count();
    assert!(n_ticks > 1, "has tick logs");

    assert!(!has_error_log());
}
