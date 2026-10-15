use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use hsd::format::source::Source;
use hsd_cli::fmt;

/// The unformatted source `fmt` should restore, and the formatted form it
/// should restore it to.
///
/// Derived from the serializer rather than hardcoded, so the test does not
/// pin the authoring shape: a newline is injected where the formatter will
/// remove it.
fn sources() -> (String, String) {
    let formatted = Source::default().to_ron().expect("serialize source");
    let mut unformatted = formatted.clone();
    unformatted.insert(1, '\n');
    (unformatted, formatted)
}

fn case_dir(case: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(case);
    // A file left by a previous run would make the idempotence check lie.
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create case dir");
    dir
}

#[test]
fn formats_and_is_idempotent() {
    let dir = case_dir("fmt-idempotent");
    let file = dir.join("asset.hsda");
    let (unformatted, formatted) = sources();
    fs::write(&file, &unformatted).expect("write source");

    let changed = fmt::run(std::slice::from_ref(&dir), true).expect("format");
    assert_eq!(changed, vec![file.clone()]);
    assert_eq!(
        fs::read_to_string(&file).expect("read formatted"),
        formatted
    );

    let changed = fmt::run(std::slice::from_ref(&dir), true).expect("reformat");
    assert!(changed.is_empty(), "second run must be a no-op");
}

#[test]
fn check_reports_without_writing() {
    let dir = case_dir("fmt-check");
    let file = dir.join("asset.hsda");
    let (unformatted, _) = sources();
    fs::write(&file, &unformatted).expect("write source");

    let changed = fmt::run(std::slice::from_ref(&dir), false).expect("check");
    assert_eq!(changed, vec![file.clone()]);
    assert_eq!(fs::read_to_string(&file).expect("read source"), unformatted);
}

/// A directory is walked for `.hsda` only; a sibling file is left alone.
#[test]
fn walk_skips_non_hsda_files() {
    let dir = case_dir("fmt-walk");
    let hsda = dir.join("asset.hsda");
    let other = dir.join("notes.txt");
    let (unformatted, _) = sources();
    fs::write(&hsda, &unformatted).expect("write source");
    fs::write(&other, &unformatted).expect("write notes");

    let changed = fmt::run(std::slice::from_ref(&dir), true).expect("format");
    assert_eq!(changed, vec![hsda]);
    assert_eq!(fs::read_to_string(&other).expect("read notes"), unformatted);
}
