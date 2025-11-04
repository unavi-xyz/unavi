use std::path::PathBuf;

use anyhow::Context;

pub fn ensure_model_assets() -> anyhow::Result<()> {
    let assets_dir = get_assets_dir()?;
    let models_dir = assets_dir.join("models");
    std::fs::create_dir_all(&models_dir).context("create models directory")?;

    let files = ["default.vrm", "animations.glb"];

    for filename in files {
        let path = models_dir.join(filename);
        if !path.exists() {
            download_asset(filename, &path)?;
        }
    }

    Ok(())
}

fn get_assets_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe().context("get current exe")?;
    let exe_dir = exe
        .parent()
        .ok_or(anyhow::anyhow!("exe has no parent directory"))?;

    Ok(exe_dir.join("assets"))
}

fn download_asset(filename: &str, dest_path: &PathBuf) -> anyhow::Result<()> {
    println!("Downloading {filename}...");

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

    println!("Downloaded {filename}");
    Ok(())
}
