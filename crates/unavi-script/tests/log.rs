use setup::{
    construct_script,
    logs::{LOGS, has_error_log},
    tick_app,
};

mod setup;

#[test]
fn script_log() {
    let mut app = setup::setup_test_app("log");
    construct_script(&mut app);

    for _ in 0..5 {
        tick_app(&mut app);
    }

    let n_inits = LOGS
        .logs
        .lock()
        .expect("test value expected")
        .iter()
        .filter(|line| line.contains("test:log") && line.contains("hello from init"))
        .count();

    assert_eq!(n_inits, 1, "has 1 startup log");

    let n_ticks = LOGS
        .logs
        .lock()
        .expect("test value expected")
        .iter()
        .filter(|line| line.contains("test:log") && line.contains("hello from tick"))
        .count();
    assert!(n_ticks > 1, "has tick logs");

    assert!(!has_error_log());
}
