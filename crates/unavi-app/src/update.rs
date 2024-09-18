use std::process::Command;

use anyhow::{bail, Result};
use self_update::{backends::github::ReleaseList, cargo_crate_version};
use tempfile::tempdir_in;
use zip::ZipArchive;

use crate::dirs::get_project_dirs;

pub fn check_for_updates() -> Result<()> {
    println!("Checking for updates...");

    // Get latest GitHub release.
    let target = env!("TARGET");
    let pkg = env!("CARGO_PKG_NAME");

    let target = if target.contains("linux") {
        "linux"
    } else if target.contains("windows") {
        "windows"
    } else if target.contains("darwin") {
        "macos"
    } else {
        panic!("Unknown target");
    };

    let releases = ReleaseList::configure()
        .repo_owner("unavi-xyz")
        .repo_name("unavi")
        .build()?
        .fetch()?;

    let release = releases[0].clone();

    if release.version == cargo_crate_version!() {
        println!("No updates found!");
        return Ok(());
    }

    println!(
        "Update found! {} -> {}",
        cargo_crate_version!(),
        release.version
    );

    let Some(asset) = release.asset_for(target, Some(pkg)) else {
        bail!("Release asset not found!");
    };

    let dirs = get_project_dirs();
    let tmp_dir = tempdir_in(dirs.data_dir())?;
    let tmp_dir_path = tmp_dir.path();
    let tmp_zip_path = tmp_dir_path.join(&asset.name);
    let tmp_zip = std::fs::File::create(&tmp_zip_path)?;

    // Download asset.
    println!("Downloading {}", &asset.download_url);
    self_update::Download::from_url(&asset.download_url)
        .set_header(reqwest::header::ACCEPT, "application/octet-stream".parse()?)
        .download_to(&tmp_zip)?;

    // Extract archive.
    let tmp_zip = std::fs::File::open(&tmp_zip_path)?;
    ZipArchive::new(tmp_zip)?.extract(tmp_dir_path)?;

    // Replace binary.
    let exe_dir = std::env::current_exe()?.parent().unwrap().to_owned();
    let bin_name = std::path::PathBuf::from(pkg);
    let new_bin = tmp_dir_path.join(bin_name);
    self_update::self_replace::self_replace(new_bin)?;
    std::fs::remove_file(tmp_zip_path)?;

    // Replace other files.
    for entry in std::fs::read_dir(tmp_dir_path)? {
        let entry = entry?;
        let file_name = entry.file_name();

        if file_name == pkg {
            continue;
        }

        let out_path = exe_dir.join(file_name);

        if entry.file_type()?.is_dir() {
            std::fs::remove_dir_all(&out_path)?;
        }

        std::fs::rename(entry.path(), out_path)?;
    }

    println!("Update successful!");

    // Run the new binary.
    let mut args = std::env::args();
    let cmd = args.next().unwrap();
    let args = args.collect::<Vec<_>>();

    drop(tmp_dir);

    Command::new(cmd)
        .args(args)
        .spawn()
        .expect("Failed to spawn new process")
        .wait()
        .unwrap();
    std::process::exit(0);
}
