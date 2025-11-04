use std::path::{Path, PathBuf};

use anyhow::Context;
use bevy::log::info;

use crate::{assets_dir, models_dir};

/// Copy bundled assets from the relative asset directory to DIRS location.
/// In dev: copies from CARGO_MANIFEST_DIR/assets
/// In release: copies from exe_dir/assets
pub fn copy_assets_to_dirs() -> anyhow::Result<()> {
    let source_dir = get_relative_assets_dir()?;

    if !source_dir.exists() {
        info!("source asset directory not found, skipping copy");
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

pub fn ensure_downloaded_assets() -> anyhow::Result<()> {
    let models = models_dir();
    std::fs::create_dir_all(&models).context("create models directory")?;

    let files = ["default.vrm", "animations.glb"];

    let missing_files: Vec<_> = files
        .iter()
        .filter(|filename| !models.join(filename).exists())
        .copied()
        .collect();

    if missing_files.is_empty() {
        return Ok(());
    }

    std::thread::scope(|s| {
        let handles: Vec<_> = missing_files
            .iter()
            .map(|filename| {
                let path = models.join(filename);
                s.spawn(move || download_asset(filename, &path))
            })
            .collect();

        for handle in handles {
            handle.join().unwrap()?;
        }

        Ok(())
    })
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
        .ok_or(anyhow::anyhow!("exe has no parent directory"))?;

    Ok(exe_dir.join("assets"))
}

fn download_asset(filename: &str, dest_path: &PathBuf) -> anyhow::Result<()> {
    info!("Downloading {filename}...");

    let url = format!("{}{}", unavi_constants::MODELS_URL, filename);
    let client = reqwest::blocking::Client::new();

    let response = client
        .get(&url)
        .send()
        .context("failed to download asset")?;

    if !response.status().is_success() {
        anyhow::bail!("download failed with status: {}", response.status());
    }

    let bytes = response.bytes().context("failed to read response bytes")?;
    std::fs::write(dest_path, bytes).context("failed to write asset file")?;

    info!("Downloaded {filename}");
    Ok(())
}
