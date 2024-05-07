use std::path::Path;

use anyhow::{anyhow, Result};
use self_update::{backends::github::ReleaseList, cargo_crate_version, self_replace::self_replace};
use semver::Version;
use zip::ZipArchive;

const ASSETS_DIR: &str = "assets";
const BIN_NAME: &str = "unavi-app";
const UPDATE_DIR: &str = ".unavi/app/update";

/// Check for new Github releases and update if one is found.
pub fn check_for_updates(force: bool) -> Result<()> {
    println!("Checking for updates...");

    let mut target = self_update::get_target();

    if target.contains("x86_64") && target.contains("linux") {
        target = "x86_64-linux"
    }
    if target.contains("x86_64") && target.contains("darwin") {
        target = "x86_64-darwin"
    }
    if target.contains("x86_64") && target.contains("windows") {
        target = "x86_64-windows"
    }

    let releases = ReleaseList::configure()
        .repo_owner("unavi-xyz")
        .repo_name("unavi")
        .build()?
        .fetch()?;

    let asset = releases
        .first()
        .ok_or(anyhow!("No release found"))?
        .asset_for(target, Some(BIN_NAME))
        .ok_or(anyhow!("No release found"))?;

    let current_version = Version::parse(cargo_crate_version!())?;
    println!("Current version: {}", current_version);

    let release_str = asset
        .name
        .strip_suffix(".zip")
        .ok_or(anyhow!("Invalid release name: {}", asset.name))?;

    let release_str = release_str
        .split('-')
        .last()
        .ok_or(anyhow!("Invalid release name: {}", asset.name))?;

    let release_version = Version::parse(release_str)?;
    println!("Latest version: {}", release_version);

    if !force && (release_version <= current_version) {
        return Ok(());
    }

    println!("Updating...");

    let _ = std::fs::remove_dir_all(UPDATE_DIR);
    std::fs::create_dir_all(UPDATE_DIR)?;

    let tmp_dir = Path::new(UPDATE_DIR);
    let tmp_zip_path = tmp_dir.join(&asset.name);
    let tmp_zip = std::fs::File::create(&tmp_zip_path)?;

    self_update::Download::from_url(&asset.download_url)
        .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?)
        .download_to(&tmp_zip)?;

    let tmp_zip = std::fs::File::open(tmp_zip_path)?;
    let mut archive = ZipArchive::new(tmp_zip)?;

    let tmp_archive_path = tmp_dir.join("archive");
    archive.extract(&tmp_archive_path)?;

    std::fs::remove_dir_all(ASSETS_DIR)?;
    let new_assets = tmp_archive_path.join(ASSETS_DIR);
    std::fs::rename(new_assets, ASSETS_DIR)?;

    let new_bin = tmp_archive_path.join(BIN_NAME);
    self_replace(new_bin)?;

    std::fs::remove_dir_all(UPDATE_DIR)?;

    let exe = std::env::current_exe()?;
    std::process::Command::new(exe).spawn()?;

    Ok(())
}
