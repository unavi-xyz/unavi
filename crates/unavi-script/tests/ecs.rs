use setup::{
    construct_script,
    logs::{count_logs_with, has_error_log, has_log},
    tick_app,
};

mod setup;

#[test]
fn script_ecs() {
    let mut app = setup::setup_test_app("ecs");
    construct_script(&mut app);

    // Startup
    tick_app(&mut app);
    assert!(has_log("startup_system"));
    assert!(!has_log("update_"));

    // Update
    for i in 1..=5 {
        tick_app(&mut app);
        assert_eq!(count_logs_with("update_1"), i);
        assert_eq!(count_logs_with("update_2"), i);
    }

    assert!(!has_error_log());
}

// TODO: Test VComponent cleanup
// TODO: Test VEntity cleanup
