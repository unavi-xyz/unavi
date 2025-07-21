use std::{path::PathBuf, time::Duration};

mod setup;

#[test]
fn script_invalid() {
    let mut app = setup::setup_test_app("invalid");

    // Write an invalid wasm asset.
    let bytes = [0; 128];
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../unavi/assets/wasm/test/invalid.wasm");
    std::fs::write(path, bytes).expect("write file");

    // Load script asset.
    app.update();
    std::thread::sleep(Duration::from_millis(200));

    // Instantiate script wasm.
    // This should error, but not panic.
    app.update();
    std::thread::sleep(Duration::from_millis(200));
    app.update();

    assert!(
        setup::LOGS
            .logs
            .lock()
            .unwrap()
            .iter()
            .any(|line| line.contains("Error loading script"))
    );
}
