use std::{
    collections::HashMap,
    path::PathBuf,
};

use anyhow::{
    Context,
    Result,
    bail,
};
use clap::{
    Args,
    Parser,
};
use hsd_cli::{
    compile,
    dump,
    fmt,
};

#[derive(Parser, Debug)]
#[command(version)]
enum HsdCli {
    Build(Build),
    Dump(Dump),
    #[command(alias = "format")]
    Fmt(Fmt),
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

/// Pretty-print `.hsda` sources in place.
///
/// A path is either a `.hsda` file or a directory, which is walked for
/// `.hsda` files. `--check` writes nothing and exits non-zero when any file
/// would change.
#[derive(Args, Debug)]
struct Fmt {
    /// `.hsda` files or directories to format
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
    /// Report what would change without writing it
    #[arg(long)]
    check:  bool,
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
            let out = out_dir.join(format!("{name}.{}", hsd::format::package::EXTENSION));
            std::fs::write(&out, package.encode()?)
                .with_context(|| format!("writing {}", out.display()))?;
            println!("wrote {}", out.display());
        }
        HsdCli::Dump(Dump { input }) => {
            println!("{}", dump::dump_file(&input)?);
        }
        HsdCli::Fmt(Fmt { inputs, check }) => {
            let changed = fmt::run(&inputs, !check)?;

            if check {
                for file in &changed {
                    eprintln!("would reformat {}", file.display());
                }
                if !changed.is_empty() {
                    bail!(
                        "{} file(s) need formatting; run `hsd-cli fmt`",
                        changed.len()
                    );
                }
            } else {
                for file in &changed {
                    println!("formatted {}", file.display());
                }
            }
        }
    }
    Ok(())
}
