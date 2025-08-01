use setup::{construct_script, logs::has_error_log, tick_app};

mod setup;

#[test]
fn script_ecs() {
    let mut app = setup::setup_test_app("ecs");
    construct_script(&mut app);

    // Startup
    tick_app(&mut app);

    // Update
    tick_app(&mut app);

    assert!(!has_error_log());
}

// TODO: Test VComponent cleanup
// TODO: Test VEntity cleanup
