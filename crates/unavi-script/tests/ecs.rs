use std::time::Duration;

use setup::construct_script;

mod setup;

#[test]
fn script_ecs() {
    let mut app = setup::setup_test_app("ecs");

    construct_script(&mut app);

    // Execute script.
    app.update();
    std::thread::sleep(Duration::from_millis(100));
    app.update();
    std::thread::sleep(Duration::from_millis(100));
    app.update();
}

// TODO: Test VComponent cleanup
// TODO: Test VSystem cleanup
