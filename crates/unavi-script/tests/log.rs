use std::time::Duration;

mod setup;

#[test]
fn script_log() {
    let mut app = setup::setup_test_app("log");

    // Load script asset.
    app.update();
    std::thread::sleep(Duration::from_millis(200));

    // Instantiate script wasm.
    app.update();
    std::thread::sleep(Duration::from_millis(200));

    // Execute script constructor.
    app.update();
    std::thread::sleep(Duration::from_millis(100));

    // Execute script update.
    app.update();
    std::thread::sleep(Duration::from_millis(100));

    // Flush logs.
    app.update();

    assert_eq!(
        setup::LOGS
            .logs
            .lock()
            .unwrap()
            .iter()
            .filter(|line| line.contains("[test:log] hello from startup"))
            .count(),
        1
    );
    assert!(
        setup::LOGS
            .logs
            .lock()
            .unwrap()
            .iter()
            .any(|line| line.contains("[test:log] hello from update"))
    );
}
