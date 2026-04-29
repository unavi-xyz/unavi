use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use blake3::Hash;

use crate::{
    blobs::write_blob,
    cargo::{derive_name, find_lib_deps, read_cargo_name},
};

pub fn build_wasm_for_crate(
    crate_dir: &Path,
    out_dir: &Path,
    built: &mut BTreeMap<String, Hash>,
) -> Result<Hash> {
    let crate_dir = std::fs::canonicalize(crate_dir)
        .with_context(|| format!("resolving crate dir {}", crate_dir.display()))?;
    let cargo_toml = crate_dir.join("Cargo.toml");
    let crate_name = read_cargo_name(&cargo_toml)?;
    let output_name = derive_name(&crate_name);

    if let Some(hash) = built.get(&output_name) {
        return Ok(*hash);
    }

    let lib_deps = find_lib_deps(&crate_dir)?;
    let mut dep_hashes: Vec<(String, Hash)> = Vec::with_capacity(lib_deps.len());
    for (dep_name, dep_crate_dir) in &lib_deps {
        let dep_hash = build_wasm_for_crate(dep_crate_dir, out_dir, built)?;
        dep_hashes.push((dep_name.clone(), dep_hash));
    }

    let wasm_file_name = format!("{}.wasm", crate_name.replace('-', "_"));
    let target_dir = PathBuf::from("target").join(&output_name);
    let src = target_dir
        .join("wasm32-wasip2")
        .join("release-wasm")
        .join(&wasm_file_name);

    let tmp_dir = out_dir.join(".tmp");
    std::fs::create_dir_all(&tmp_dir).with_context(|| format!("creating {}", tmp_dir.display()))?;
    let dst = tmp_dir.join(format!("{output_name}.wasm"));

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
            "crates/unavi-script/node_modules/@bytecodealliance/jco/lib/wasi_snapshot_preview1.reactor.wasm",
            "-o",
            dst.to_str().context("component out")?,
        ],
    )
    .context("wasm-tools component new")?;

    for (dep_name, dep_hash) in &dep_hashes {
        let dep_wasm = out_dir.join(dep_hash.to_string());
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
            .with_context(|| format!("running wac plug {dep_name}"))?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            if err.contains("no matching imports") {
                continue;
            }
            anyhow::bail!("wac plug {dep_name}: {err}");
        }
    }

    let bytes = std::fs::read(&dst).context("read built wasm")?;
    let hash = write_blob(out_dir, &bytes)?;
    std::fs::remove_file(&dst).ok();

    built.insert(output_name, hash);

    Ok(hash)
}

pub fn run_cmd(program: &str, args: &[&str]) -> Result<()> {
    let status = std::process::Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("running {program}"))?;
    ensure!(status.success(), "{program} exited with {status}");
    Ok(())
}
