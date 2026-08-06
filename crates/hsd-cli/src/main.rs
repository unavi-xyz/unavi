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
use hsd::source::Source;
use hsd_cli::{
    compile,
    dump,
};

#[derive(Parser, Debug)]
#[command(version)]
enum HsdCli {
    Build(Build),
    Dump(Dump),
    Format(Format),
}

/// Compile a `.hsda` source into a single `.hsdz` package.
#[derive(Args, Debug)]
struct Build {
    /// Input `.hsda` file path
    #[arg(short, long)]
    input:   PathBuf,
    /// Output directory for the compiled `.hsdz`
    #[arg(short, long)]
    out_dir: PathBuf,
}

/// Print a compiled `.hsdz` as `.hsda`-shaped RON.
///
/// An inspection view, not source: compilation replaced paths with content.
#[derive(Args, Debug)]
struct Dump {
    /// `.hsdz` file to inspect
    input: PathBuf,
}

/// Pretty-print a `.hsda` source file in place.
#[derive(Args, Debug)]
struct Format {
    /// `.hsda` file to format
    input: PathBuf,
}

fn main() -> Result<()> {
    match HsdCli::parse() {
        HsdCli::Build(Build { input, out_dir }) => {
            std::fs::create_dir_all(&out_dir)
                .with_context(|| format!("creating {}", out_dir.display()))?;

            let input_abs = std::fs::canonicalize(&input)
                .with_context(|| format!("resolving {}", input.display()))?;

            let mut built = HashMap::new();
            let package = compile::compile_file(&input_abs, &mut built)?;

            let name = compile::output_name(&input_abs);
            let out = out_dir.join(format!("{name}.{}", hsd::package::EXTENSION));
            std::fs::write(&out, package.encode()?)
                .with_context(|| format!("writing {}", out.display()))?;
            println!("wrote {}", out.display());
        }
        HsdCli::Dump(Dump { input }) => {
            println!("{}", dump::dump_file(&input)?);
        }
        HsdCli::Format(Format { input }) => {
            let src = std::fs::read_to_string(&input)
                .with_context(|| format!("reading {}", input.display()))?;
            let doc =
                Source::parse(&src).with_context(|| format!("parsing {}", input.display()))?;
            std::fs::write(&input, doc.to_ron()?)
                .with_context(|| format!("writing {}", input.display()))?;
        }
    }
    Ok(())
}
