use std::path::PathBuf;

use setup::{construct_script, logs::LOGS};

mod setup;

#[test]
fn script_invalid() {
    let mut app = setup::setup_test_app("invalid");

    // Write an invalid wasm asset.
    let bytes = [0; 128];
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../unavi/assets/wasm/test/invalid.wasm");
    std::fs::write(path, bytes).expect("write file");

    // This should error, but not panic.
    construct_script(&mut app);
    assert_eq!(
        LOGS.logs
            .lock()
            .unwrap()
            .iter()
            .filter(|line| line.contains("Error instantiating script component"))
            .count(),
        1
    );
}
