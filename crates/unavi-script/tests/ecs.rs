use setup::{
    construct_script,
    logs::{has_error_log, has_log},
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

    // Update
    tick_app(&mut app);
    assert!(has_log("read_system"));
    assert!(has_log("write_system"));

    assert!(!has_error_log());
}

// TODO: Test VComponent cleanup
// TODO: Test VEntity cleanup
