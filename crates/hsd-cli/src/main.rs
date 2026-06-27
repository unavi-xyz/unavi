use std::{
    collections::HashMap,
    path::PathBuf,
};

use anyhow::{
    Context,
    Result,
};
use clap::{
    Args,
    Parser,
};
use hsd_cli::{
    build,
    format,
};

#[derive(Parser, Debug)]
#[command(version)]
enum HsdCli {
    Build(Build),
    Format(Format),
}

/// Compile an HSDX source into a flat output directory.
#[derive(Args, Debug)]
struct Build {
    /// Input HSDX file path
    #[arg(short, long)]
    input:   PathBuf,
    /// Output directory for the compiled HSD and WASM files
    #[arg(short, long)]
    out_dir: PathBuf,
}

/// Pretty-print an HSD or HSDX file using RON pretty formatting.
#[derive(Args, Debug)]
struct Format {
    /// HSD or HSDX file to format
    input: PathBuf,
}

fn main() -> Result<()> {
    match HsdCli::parse() {
        HsdCli::Build(Build { input, out_dir }) => {
            std::fs::create_dir_all(&out_dir)
                .with_context(|| format!("creating {}", out_dir.display()))?;
            let mut built = HashMap::new();
            build::build_hsdx_to_hsd(&input, &out_dir, &mut built)?;
        }
        HsdCli::Format(Format { input }) => {
            format::format_file(&input)?;
        }
    }
    Ok(())
}
