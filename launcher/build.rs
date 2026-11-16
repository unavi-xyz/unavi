use std::{
    env,
    fs,
    path::Path,
};

use base64::Engine;

/// Relative to the crate root for a local checkout.
const DEFAULT_ASSETS: &str = "../assets";

fn main() {
    // Nix builds root the source at the launcher workspace, so the shared
    // brand assets are passed in by path; local builds sit beside them.
    let assets = env::var("UNAVI_ASSETS").unwrap_or_else(|_| DEFAULT_ASSETS.to_string());
    let assets = Path::new(&assets);

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let out_dir = Path::new(&out_dir);

    let logo = assets.join("icon-nobg.png");
    println!("cargo:rerun-if-changed={}", logo.display());
    let png = fs::read(&logo).expect("read logo");
    let encoded = base64::engine::general_purpose::STANDARD.encode(png);
    fs::write(
        out_dir.join("logo.uri"),
        format!("data:image/png;base64,{encoded}"),
    )
    .expect("write logo data uri");

    let icon = assets.join("icon-rounded.png");
    println!("cargo:rerun-if-changed={}", icon.display());
    fs::copy(&icon, out_dir.join("icon-rounded.png")).expect("copy icon");
}
