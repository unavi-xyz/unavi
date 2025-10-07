use setup::{construct_script, logs::has_error_log, tick_app};

mod setup;

#[test]
fn script_scene() {
    let mut app = setup::setup_test_app("scene");
    construct_script(&mut app);

    // Startup
    tick_app(&mut app);

    // Update
    for _ in 1..=5 {
        tick_app(&mut app);
    }

    assert!(!has_error_log());
}
