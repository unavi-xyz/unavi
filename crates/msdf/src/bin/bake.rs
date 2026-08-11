//! Bakes a font into the distance field the client embeds.
//!
//! Run when the charset, the font, or the field parameters change:
//! `cargo run --bin msdf-bake --features bake`.

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use msdf::{
    atlas::LATIN,
    bake::{
        BakeOpts,
        DEFAULT_FONT,
        bake,
    },
};

#[derive(Parser)]
struct Args {
    /// Font to bake. Defaults to the bundled Noto Sans Regular.
    #[arg(long)]
    font:         Option<PathBuf>,
    /// Where to write the field and its metrics. The client bakes its own
    /// through a build script; this is for judging parameters by eye.
    #[arg(long, default_value = "target/msdf")]
    out:          PathBuf,
    #[arg(long, default_value = "noto-sans-latin")]
    name:         String,
    #[arg(long, default_value_t = 32)]
    px_per_em:    u32,
    #[arg(long, default_value_t = 6.0)]
    range:        f64,
    /// Characters to cover, one file of them. Defaults to the Latin set.
    #[arg(long)]
    charset_file: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let font = match &args.font {
        Some(path) => std::fs::read(path).with_context(|| format!("read {}", path.display()))?,
        None => DEFAULT_FONT.to_vec(),
    };
    let charset = match &args.charset_file {
        Some(path) => {
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?
        }
        None => LATIN.to_string(),
    };

    let baked = bake(
        &font,
        &BakeOpts {
            px_per_em: args.px_per_em,
            range: args.range,
            charset,
            ..Default::default()
        },
    )?;

    std::fs::create_dir_all(&args.out)?;
    let png = args.out.join(format!("{}.png", args.name));
    let metrics = args.out.join(format!("{}.bin", args.name));
    baked.image.save(&png)?;
    std::fs::write(&metrics, baked.atlas.encode()?)?;

    println!(
        "{}x{} atlas, {} glyphs, {} kerning pairs",
        baked.atlas.width,
        baked.atlas.height,
        baked.atlas.glyphs.len(),
        baked.atlas.kerning.len(),
    );
    println!(
        "{} ({} KiB), {}",
        png.display(),
        std::fs::metadata(&png)?.len() / 1024,
        metrics.display(),
    );
    if !baked.missing.is_empty() {
        println!(
            "the face has no glyph for: {}",
            baked.missing.iter().collect::<String>()
        );
    }

    Ok(())
}
