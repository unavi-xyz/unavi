use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result};
use blake3::Hash;
use hsd::{Hsd, HsdImage, HsdNode, Hsdx};

use crate::{blobs::write_blob, wasm::build_wasm_for_crate};

pub fn build_hsdx_to_hsd(
    input: &Path,
    out_dir: &Path,
    built: &mut BTreeMap<String, Hash>,
) -> Result<Hash> {
    let input_abs =
        std::fs::canonicalize(input).with_context(|| format!("resolving {}", input.display()))?;
    let input_dir = input_abs.parent().context("input has no parent dir")?;

    let src = std::fs::read_to_string(&input_abs)
        .with_context(|| format!("reading {}", input_abs.display()))?;
    let hsdx = Hsdx::parse(&src).with_context(|| format!("parsing {}", input_abs.display()))?;

    let mut hsd = Hsd {
        materials: hsdx.materials,
        meshes: hsdx.meshes,
        ..Default::default()
    };

    // Assets
    for (key, asset_ref) in hsdx.assets {
        if asset_ref.ends_with(".hsdx") {
            let dep_path = input_dir.join(&*asset_ref);
            let dep_hash = build_hsdx_to_hsd(&dep_path, out_dir, built)?;
            hsd.assets.insert(key, dep_hash.into());
        }
    }

    // Nodes
    for (key, node) in hsdx.nodes {
        let mut out_scripts = Vec::new();

        if let Some(scripts) = &node.scripts {
            for script in scripts {
                if !script.ends_with("Cargo.toml") {
                    continue;
                }
                let cargo_path = input_dir.join(script);
                let crate_dir = cargo_path
                    .parent()
                    .context("Cargo.toml has no parent dir")?;
                let hash = build_wasm_for_crate(crate_dir, out_dir, built)?;
                out_scripts.push(hash.into());
            }
        }

        let out_node = HsdNode::from_hsdx(node, Some(out_scripts));
        hsd.nodes.insert(key, out_node);
    }

    // Images
    for (key, img) in hsdx.images {
        let Some(path) = &img.data else {
            continue;
        };
        let abs = std::fs::canonicalize(input_dir.join(path))
            .with_context(|| format!("resolving image path {path}"))?;
        let bytes =
            std::fs::read(&abs).with_context(|| format!("reading image {}", abs.display()))?;
        let hash = write_blob(out_dir, &bytes)?;

        let out_img = HsdImage::from_hsdx(img, Some(hash.into()));
        hsd.images.insert(key, out_img);
    }

    // The compiled HSD is itself a blob (referenced by hash from parent HSDs).
    // This is a little weird, but whatever..
    let bytes = hsd.to_ron()?.into_bytes();
    let hash = write_blob(out_dir, &bytes)?;

    // Also write a named copy at the top level for entry-point loads.
    let crate_dir_name = input_dir
        .file_name()
        .with_context(|| format!("input dir has no name: {}", input_dir.display()))?
        .to_string_lossy();
    let output_name = crate_dir_name.replace('-', "_");
    let out_hsd = out_dir.join(format!("{output_name}.hsd"));
    std::fs::write(&out_hsd, &bytes).with_context(|| format!("writing {}", out_hsd.display()))?;
    println!("wrote {}", out_hsd.display());

    Ok(hash)
}
