use setup::{
    construct_script,
    logs::{LOGS, count_logs_with, has_error_log, has_log},
    tick_app,
};

mod setup;

const N_UPDATE_SYSTEMS: usize = 3;

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

        for n in 1..=N_UPDATE_SYSTEMS {
            assert_eq!(count_logs_with(&format!("update_{n}")), i);
        }
    }

    // Validate system order
    {
        let logs = LOGS.logs.lock().unwrap();

        let update_order = logs
            .iter()
            .filter_map(|line| {
                if line.to_lowercase().contains("update_") {
                    let num = line.split('_').collect::<Vec<_>>()[1]
                        .parse::<usize>()
                        .unwrap();
                    Some(num)
                } else {
                    None
                }
            })
            .take(N_UPDATE_SYSTEMS);

        let mut i = 1;
        println!("Update order: {update_order:?}");

        for num in update_order {
            assert_eq!(num, i);
            i += 1;
        }
    }

    assert!(!has_error_log());
}

// TODO: Test VComponent cleanup
// TODO: Test VEntity cleanup
