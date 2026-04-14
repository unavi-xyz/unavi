use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};

use crate::cargo::{derive_name, find_lib_deps, read_cargo_name};

pub fn build_wasm_for_crate(
    crate_dir: &Path,
    out_dir: &Path,
    built: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    let crate_dir = std::fs::canonicalize(crate_dir)
        .with_context(|| format!("resolving crate dir {}", crate_dir.display()))?;
    let cargo_toml = crate_dir.join("Cargo.toml");
    let crate_name = read_cargo_name(&cargo_toml)?;
    let output_name = derive_name(&crate_name);

    if built.contains(&output_name) {
        return Ok(());
    }

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

pub fn run_cmd(program: &str, args: &[&str]) -> Result<()> {
    let status = std::process::Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("running {program}"))?;
    ensure!(status.success(), "{program} exited with {status}");
    Ok(())
}
