use setup::{construct_script, logs::count_logs_with};

mod setup;

#[test]
fn script_invalid() {
    let mut app = setup::setup_test_app("invalid", Some(vec![0; 128]));

    // This should error, but not panic.
    construct_script(&mut app);

    assert_eq!(count_logs_with("error instantiating script component"), 1);
}
