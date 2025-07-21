use std::time::Duration;

mod setup;

#[test]
fn script_stall() {
    let mut app = setup::setup_test_app("stall");

    // Load script asset.
    app.update();
    std::thread::sleep(Duration::from_millis(200));

    // Instantiate script wasm.
    app.update();
    std::thread::sleep(Duration::from_millis(200));

    // Execute script constructor.
    app.update();

    // Execute script updates.
    // Should never complete, but the ECS should go on fine.
    app.update();
    app.update();
    app.update();

    assert_eq!(
        setup::LOGS
            .logs
            .lock()
            .unwrap()
            .iter()
            .filter(|line| line.contains("[test:stall] new"))
            .count(),
        1
    );
    assert!(
        !setup::LOGS
            .logs
            .lock()
            .unwrap()
            .iter()
            .any(|line| line.contains("[test:stall] update"))
    );
}
