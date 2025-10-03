use setup::{
    construct_script,
    logs::{LOGS, count_logs_with, has_error_log, has_log},
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

    // Validate system order
    {
        let logs = LOGS.logs.lock().unwrap();

        let first_update = logs
            .iter()
            .find(|line| line.to_lowercase().contains("update_"))
            .unwrap();
        let first_num = first_update.split('_').collect::<Vec<_>>()[1]
            .parse::<usize>()
            .unwrap();

        assert_eq!(first_num, 1)
    }

    assert!(!has_error_log());
}

// TODO: Test VComponent cleanup
// TODO: Test VEntity cleanup
