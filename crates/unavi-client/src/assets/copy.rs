use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::assets_dir;

/// Copy bundled assets from the relative asset directory to state location.
///
/// In dev: copies from `CARGO_MANIFEST_DIR/assets`
/// In release: copies from `exe_dir/assets`
///
/// # Errors
///
/// Returns an error if directory creation, reading, or file copying fails.
pub fn copy_assets_to_dirs() -> anyhow::Result<()> {
    let source_dir = get_relative_assets_dir()?;

    if !source_dir.exists() {
        println!("source asset directory not found, skipping copy");
        return Ok(());
    }

    let dest_dir = assets_dir();
    std::fs::create_dir_all(&dest_dir).context("create assets directory")?;

    copy_dir_recursive(&source_dir, &dest_dir)?;

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst).context("create destination directory")?;
    }

    for entry in std::fs::read_dir(src).context("read source directory")? {
        let entry = entry.context("read directory entry")?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            // only copy if dest doesn't exist or source is newer
            let should_copy = if dest_path.exists() {
                let src_modified = std::fs::metadata(&path)
                    .context("get source metadata")?
                    .modified()
                    .context("get source modified time")?;
                let dst_modified = std::fs::metadata(&dest_path)
                    .context("get dest metadata")?
                    .modified()
                    .context("get dest modified time")?;
                src_modified > dst_modified
            } else {
                true
            };

            if should_copy {
                std::fs::copy(&path, &dest_path).context("copy file")?;
            }
        }
    }

    Ok(())
}

fn get_relative_assets_dir() -> anyhow::Result<PathBuf> {
    // Development: use cargo manifest dir if available
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        return Ok(PathBuf::from(manifest_dir).join("assets"));
    }

    // Release: use exe directory
    let exe = std::env::current_exe().context("get current exe")?;
    let exe_dir = exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("exe has no parent directory"))?;

    Ok(exe_dir.join("assets"))
}
