use std::time::Duration;

mod setup;

#[test]
fn script_ecs() {
    let mut app = setup::setup_test_app("ecs");

    // Load script asset.
    app.update();
    std::thread::sleep(Duration::from_millis(200));

    // Instantiate script wasm.
    app.update();
    std::thread::sleep(Duration::from_millis(200));

    // Execute script.
    app.update();
    std::thread::sleep(Duration::from_millis(100));
    app.update();
    std::thread::sleep(Duration::from_millis(100));
    app.update();
}
