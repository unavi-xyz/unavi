use std::path::Path;

use anyhow::Context;
use unavi_player::{animation::defaults::DEFAULT_ANIMATIONS, avatar_spawner::DEFAULT_AVATAR};

use crate::assets_dir;

const CDN_ASSETS_URL: &str = "https://unavi.nyc3.cdn.digitaloceanspaces.com/assets";

/// Download web assets to local storage if they don't already exist.
///
/// # Errors
///
/// Returns an error if directory creation or download fails.
pub fn download_web_assets() -> anyhow::Result<()> {
    std::fs::create_dir_all(assets_dir()).context("create assets directory")?;

    let assets = [DEFAULT_AVATAR, DEFAULT_ANIMATIONS];

    for asset_path in assets {
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

        let url = format!("{CDN_ASSETS_URL}/{asset_path}");

        if let Err(e) = download_file(&url, &dest_path) {
            eprintln!("failed to download {asset_path}: {e:?}");
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
