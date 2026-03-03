use setup::{
    construct_script,
    logs::{LOGS, has_error_log},
    tick_app,
};

mod setup;

#[test]
fn script_stall() {
    let mut app = setup::setup_test_app("stall", None);
    construct_script(&mut app);

    assert_eq!(
        LOGS.logs
            .lock()
            .expect("test value expected")
            .iter()
            .filter(|line| line.contains("hello from init"))
            .count(),
        1
    );

    // Execute script tick.
    // Should never complete, but the Bevy ECS should go on fine.
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
