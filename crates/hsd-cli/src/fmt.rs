//! `hsd-cli fmt`: pretty-print `.hsda` sources in place.
//!
//! Formatting is an explicit command rather than a build side effect, so
//! building assets never rewrites a working tree. A path is either a `.hsda`
//! file or a directory, which is walked for `.hsda` files.

use std::{
    ffi::OsStr,
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use anyhow::{
    Context,
    Result,
};
use hsd::format::source::{
    EXTENSION,
    Source,
};

/// Formats every `.hsda` named by `roots`, writing in place when `write` is
/// set.
///
/// Answers the paths that differed from their formatted form, in discovery
/// order. With `write` false nothing on disk is touched, which is what
/// `--check` reads.
pub fn run(roots: &[PathBuf], write: bool) -> Result<Vec<PathBuf>> {
    let mut changed = Vec::new();

    for file in collect(roots)? {
        let source =
            fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
        let doc = Source::parse(&source).with_context(|| format!("parsing {}", file.display()))?;
        let formatted = doc
            .to_ron()
            .with_context(|| format!("formatting {}", file.display()))?;

        if formatted == source {
            continue;
        }
        if write {
            fs::write(&file, &formatted).with_context(|| format!("writing {}", file.display()))?;
        }
        changed.push(file);
    }

    Ok(changed)
}

/// The `.hsda` files a run covers: an explicit file as given, a directory
/// walked recursively. Sorted and deduplicated so two overlapping roots do
/// not format one file twice.
fn collect(roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for root in roots {
        let metadata = fs::metadata(root).with_context(|| format!("reading {}", root.display()))?;
        if metadata.is_dir() {
            walk(root, &mut files)?;
        } else {
            files.push(root.clone());
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn walk(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("reading {}", dir.display()))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        let kind = entry
            .file_type()
            .with_context(|| format!("stat {}", path.display()))?;
        if kind.is_dir() {
            walk(&path, files)?;
        } else if path.extension().and_then(OsStr::to_str) == Some(EXTENSION) {
            files.push(path);
        }
    }

    Ok(())
}
