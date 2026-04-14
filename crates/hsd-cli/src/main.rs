use std::{collections::BTreeSet, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Args, Parser};
use hsd_cli::{build, format};

#[derive(Parser, Debug)]
#[command(version)]
enum HsdCli {
    Build(Build),
    Format(Format),
}

/// Compile an HSDX source into a flat output directory.
///
/// Reads `asset.hsdx`, resolves `./Cargo.toml` script refs by building the
/// WASM component (cargo → wasm-opt → wasm-tools → wac), and resolves
/// `../dep/asset.hsdx` asset refs recursively. Outputs `{name}.hsd` and
/// `{name}.wasm` flat in `--out-dir`.
#[derive(Args, Debug)]
struct Build {
    /// Input HSDX file path (e.g. wasm/unavi-gauntlet/asset.hsdx)
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for the compiled HSD and WASM files
    #[arg(short, long)]
    out_dir: PathBuf,
}

/// Pretty-print an HSD or HSDX file using RON pretty formatting.
#[derive(Args, Debug)]
struct Format {
    /// HSD or HSDX file to format
    #[arg(short, long)]
    input: PathBuf,
}

fn main() -> Result<()> {
    match HsdCli::parse() {
        HsdCli::Build(Build { input, out_dir }) => {
            std::fs::create_dir_all(&out_dir)
                .with_context(|| format!("creating {}", out_dir.display()))?;
            let mut built = BTreeSet::new();
            build::build_hsdx_to_hsd(&input, &out_dir, &mut built)?;
        }
        HsdCli::Format(Format { input }) => {
            format::format_file(&input)?;
        }
    }
    Ok(())
}
