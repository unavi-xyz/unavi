use std::path::Path;

use anyhow::Context;
use constcat::concat;
use unavi_player::{animation::defaults::DEFAULT_ANIMATIONS, avatar_spawner::DEFAULT_AVATAR_URL};

use crate::assets_dir;

/// Download web assets to local storage if they don't already exist.
///
/// # Errors
///
/// Returns an error if directory creation or download fails.
pub fn download_web_assets() -> anyhow::Result<()> {
    std::fs::create_dir_all(assets_dir()).context("create assets directory")?;

    let vrm_url = concat!(unavi_constants::CDN_ASSETS_URL, "/", DEFAULT_AVATAR_URL);
    let animations_url = concat!(unavi_constants::CDN_ASSETS_URL, "/", DEFAULT_ANIMATIONS);

    let assets = [
        (vrm_url, DEFAULT_AVATAR_URL),
        (animations_url, DEFAULT_ANIMATIONS),
    ];

    for (url, asset_path) in assets {
        let dest_path = assets_dir().join(asset_path);
        if dest_path.exists() {
            continue;
        }

        if let Some(parent) = dest_path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).context("create asset parent directory")?;
        }

        println!("downloading asset: {asset_path}");

        match download_file(url, &dest_path) {
            Ok(()) => println!("downloaded asset: {asset_path}"),
            Err(e) => {
                eprintln!("failed to download {asset_path}: {e:?}");
            }
        }
    }

    Ok(())
}

fn download_file(url: &str, dest: &Path) -> anyhow::Result<()> {
    let response =
        reqwest::blocking::get(url).with_context(|| format!("failed to download from {url}"))?;

    let bytes = response.bytes().context("failed to read response bytes")?;

    std::fs::write(dest, bytes)
        .with_context(|| format!("failed to write to {}", dest.display()))?;

    Ok(())
}
