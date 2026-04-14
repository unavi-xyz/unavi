use std::{collections::BTreeSet, path::Path};

use anyhow::{Context, Result};
use hsd::HsdFile;

use crate::cargo::{derive_name, read_cargo_name};
use crate::wasm::build_wasm_for_crate;

pub fn build_hsdx_to_hsd(
    input: &Path,
    out_dir: &Path,
    built: &mut BTreeSet<String>,
) -> Result<String> {
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
