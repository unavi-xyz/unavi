use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use clap::{Args, Parser};
use hsd::HsdFile;

#[derive(Parser, Debug)]
#[command(version)]
enum HsdCli {
    Build(Build),
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

fn main() -> Result<()> {
    match HsdCli::parse() {
        HsdCli::Build(Build { input, out_dir }) => {
            std::fs::create_dir_all(&out_dir)
                .with_context(|| format!("creating {}", out_dir.display()))?;
            let mut built = BTreeSet::new();
            build_hsdx_to_hsd(&input, &out_dir, &mut built)?;
            Ok(())
        }
    }
}

fn build_hsdx_to_hsd(input: &Path, out_dir: &Path, built: &mut BTreeSet<String>) -> Result<String> {
    let input_abs =
        std::fs::canonicalize(input).with_context(|| format!("resolving {}", input.display()))?;
    let input_dir = input_abs.parent().context("input has no parent dir")?;

    let src = std::fs::read_to_string(&input_abs)
        .with_context(|| format!("reading {}", input_abs.display()))?;
    let mut hsd =
        HsdFile::parse(&src).with_context(|| format!("parsing {}", input_abs.display()))?;

    let mut output_name = String::new();

    for node in hsd.nodes.values_mut() {
        for script in &mut node.scripts {
            if script.ends_with("Cargo.toml") {
                let cargo_path = input_dir.join(&*script);
                let crate_name = read_cargo_name(&cargo_path)?;
                let name = derive_name(&crate_name);
                if output_name.is_empty() {
                    output_name.clone_from(&name);
                }
                let crate_dir = cargo_path
                    .parent()
                    .context("Cargo.toml has no parent dir")?;
                build_wasm_for_crate(crate_dir, out_dir, built)?;
                *script = format!("./{name}.wasm");
            }
        }
    }

    #[expect(clippy::case_sensitive_file_extension_comparisons)]
    for asset_ref in hsd.assets.values_mut() {
        if asset_ref.ends_with(".hsdx") {
            let dep_path = input_dir.join(&*asset_ref);
            let dep_name = build_hsdx_to_hsd(&dep_path, out_dir, built)?;
            *asset_ref = format!("./{dep_name}.hsd");
        }
    }

    if output_name.is_empty() {
        output_name = input_abs
            .file_stem()
            .map_or_else(|| "asset".to_string(), |s| s.to_string_lossy().to_string());
    }

    let out_hsd = out_dir.join(format!("{output_name}.hsd"));
    std::fs::write(&out_hsd, hsd.to_ron()?)?;
    println!("wrote {}", out_hsd.display());
    Ok(output_name)
}

fn build_wasm_for_crate(
    crate_dir: &Path,
    out_dir: &Path,
    built: &mut BTreeSet<String>,
) -> Result<()> {
    let crate_dir = std::fs::canonicalize(crate_dir)
        .with_context(|| format!("resolving crate dir {}", crate_dir.display()))?;
    let cargo_toml = crate_dir.join("Cargo.toml");
    let crate_name = read_cargo_name(&cargo_toml)?;
    let output_name = derive_name(&crate_name);

    if built.contains(&output_name) {
        return Ok(());
    }

    // Build all lib deps first (recursive).
    let lib_deps = find_lib_deps(&crate_dir)?;
    for (_, dep_crate_dir) in &lib_deps {
        build_wasm_for_crate(dep_crate_dir, out_dir, built)?;
    }

    let wasm_file_name = format!("{}.wasm", crate_name.replace('-', "_"));
    let target_dir = PathBuf::from("target").join(&output_name);
    let src = target_dir
        .join("wasm32-wasip2")
        .join("release-wasm")
        .join(&wasm_file_name);
    let dst = out_dir.join(format!("{output_name}.wasm"));

    println!("building {crate_name}");

    run_cmd(
        "cargo",
        &[
            "build",
            "--quiet",
            "--target",
            "wasm32-wasip2",
            "--profile",
            "release-wasm",
            "--manifest-path",
            cargo_toml.to_str().context("cargo toml path")?,
            "--target-dir",
            target_dir.to_str().context("target dir path")?,
        ],
    )
    .context("cargo build")?;

    run_cmd(
        "wasm-opt",
        &[
            "-O4",
            "--enable-bulk-memory-opt",
            "-ffm",
            src.to_str().context("src wasm path")?,
            "-o",
            dst.to_str().context("dst wasm path")?,
        ],
    )
    .context("wasm-opt")?;

    run_cmd(
        "wasm-tools",
        &[
            "component",
            "new",
            dst.to_str().context("component wasm")?,
            "--adapt",
            "scripts/wasi_snapshot_preview1.reactor.wasm",
            "-o",
            dst.to_str().context("component out")?,
        ],
    )
    .context("wasm-tools component new")?;

    // wac plug each lib dep in order. Skip if the component has no matching imports.
    for (dep_output_name, _) in &lib_deps {
        let dep_wasm = out_dir.join(format!("{dep_output_name}.wasm"));
        let output = std::process::Command::new("wac")
            .args([
                "plug",
                dst.to_str().context("plug target")?,
                "--plug",
                dep_wasm.to_str().context("plug dep")?,
                "-o",
                dst.to_str().context("plug out")?,
            ])
            .output()
            .with_context(|| format!("running wac plug {dep_output_name}"))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            if err.contains("no matching imports") {
                continue;
            }
            anyhow::bail!("wac plug {dep_output_name}: {err}");
        }
    }

    built.insert(output_name);
    Ok(())
}

fn find_lib_deps(crate_dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let deps_toml_path = crate_dir.join("wit/deps.toml");
    if !deps_toml_path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&deps_toml_path)?;
    let table: toml::Table = toml::from_str(&content)?;
    let mut result = Vec::new();
    for (name, value) in &table {
        if name.starts_with("wired-") {
            continue;
        }
        let path_str = value.as_str().with_context(|| format!("dep {name} path"))?;
        // Path is relative to crate_dir/wit/ (i.e. the deps.toml location).
        let dep_wit_abs = crate_dir.join("wit").join(path_str);
        let dep_wit_canon = std::fs::canonicalize(&dep_wit_abs)
            .with_context(|| format!("resolving dep {name}: {}", dep_wit_abs.display()))?;
        let dep_crate_dir = dep_wit_canon
            .parent()
            .with_context(|| format!("dep {name} wit dir has no parent"))?
            .to_path_buf();
        let dep_cargo_toml = dep_crate_dir.join("Cargo.toml");
        let dep_crate_name = read_cargo_name(&dep_cargo_toml)?;
        let dep_output_name = derive_name(&dep_crate_name);
        result.push((dep_output_name, dep_crate_dir));
    }
    Ok(result)
}

fn read_cargo_name(cargo_toml: &Path) -> Result<String> {
    let content = std::fs::read_to_string(cargo_toml)
        .with_context(|| format!("reading {}", cargo_toml.display()))?;
    let table: toml::Table = toml::from_str(&content)?;
    let name = table
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .with_context(|| format!("missing [package.name] in {}", cargo_toml.display()))?
        .to_string();
    Ok(name)
}

/// Strip the namespace prefix and convert hyphens to underscores.
///
/// `"unavi-gauntlet"` → `"gauntlet"`, `"unavi-vui-module"` → `"vui_module"`.
fn derive_name(crate_name: &str) -> String {
    crate_name
        .split_once('-')
        .map_or(crate_name, |x| x.1)
        .replace('-', "_")
}

fn run_cmd(program: &str, args: &[&str]) -> Result<()> {
    let status = std::process::Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("running {program}"))?;
    ensure!(status.success(), "{program} exited with {status}");
    Ok(())
}
