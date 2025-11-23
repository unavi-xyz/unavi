use setup::{
    construct_script,
    logs::{LOGS, has_error_log},
    tick_app,
};

mod setup;

#[test]
fn script_stall() {
    let mut app = setup::setup_test_app("stall");
    construct_script(&mut app);

    // Execute script startup.
    tick_app(&mut app);
    assert_eq!(
        LOGS.logs
            .lock()
            .expect("test value expected")
            .iter()
            .filter(|line| line.contains("test:stall") && line.contains("hello from startup"))
            .count(),
        1
    );

    // Execute script update.
    // Should never complete, but the Bevy ECS should go on fine.
    tick_app(&mut app);
    tick_app(&mut app);
    tick_app(&mut app);
    assert!(
        !LOGS
            .logs
            .lock()
            .expect("test value expected")
            .iter()
            .any(|line| line.contains("test:stall") && line.contains("hello from update"))
    );

    assert!(!has_error_log());
}
