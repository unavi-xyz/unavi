//! Bakes the shipped distance field into `OUT_DIR`.
//!
//! Generated rather than committed, for the same reason the HSD assets are:
//! a derived artifact in the tree is one more thing that can be stale. It runs
//! in about a second and only when this file or `msdf` itself changes.

use msdf::bake::{
    BakeOpts,
    DEFAULT_FONT,
    bake,
};

fn main() -> anyhow::Result<()> {
    println!("cargo::rerun-if-changed=build.rs");

    let out = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let baked = bake(DEFAULT_FONT, &BakeOpts::default())?;

    anyhow::ensure!(
        baked.missing.is_empty(),
        "the font has no glyph for {:?}; the charset and the face disagree",
        baked.missing,
    );

    baked.image.save(out.join("latin.png"))?;
    std::fs::write(out.join("latin.bin"), baked.atlas.encode()?)?;

    Ok(())
}
