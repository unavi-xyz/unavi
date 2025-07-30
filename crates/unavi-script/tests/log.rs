use setup::{construct_script, logs::LOGS, tick_app};

mod setup;

#[test]
fn script_log() {
    let mut app = setup::setup_test_app("log");

    construct_script(&mut app);

    // Execute script startup.
    tick_app(&mut app);
    assert_eq!(n_startup_logs(), 1);
    assert!(!has_update_log());

    // Execute script update.
    tick_app(&mut app);
    assert_eq!(n_startup_logs(), 1);
    assert!(has_update_log());
}

fn n_startup_logs() -> usize {
    LOGS.logs
        .lock()
        .unwrap()
        .iter()
        .filter(|line| line.contains("test:log") && line.contains("hello from startup"))
        .count()
}

fn has_update_log() -> bool {
    LOGS.logs
        .lock()
        .unwrap()
        .iter()
        .any(|line| line.contains("test:log") && line.contains("hello from update"))
}
