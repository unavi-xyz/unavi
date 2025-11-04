use std::path::PathBuf;

use anyhow::Context;
use bevy::log::info;

pub fn ensure_model_assets() -> anyhow::Result<()> {
    let assets_dir = get_assets_dir()?;
    let models_dir = assets_dir.join("models");
    std::fs::create_dir_all(&models_dir).context("create models directory")?;

    let files = ["default.vrm", "animations.glb"];

    let missing_files: Vec<_> = files
        .iter()
        .filter(|filename| !models_dir.join(filename).exists())
        .copied()
        .collect();

    if missing_files.is_empty() {
        return Ok(());
    }

    std::thread::scope(|s| {
        let handles: Vec<_> = missing_files
            .iter()
            .map(|filename| {
                let path = models_dir.join(filename);
                s.spawn(move || download_asset(filename, &path))
            })
            .collect();

        for handle in handles {
            handle.join().unwrap()?;
        }

        Ok(())
    })
}

fn get_assets_dir() -> anyhow::Result<PathBuf> {
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
