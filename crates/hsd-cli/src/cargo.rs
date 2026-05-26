use std::path::{
    Path,
    PathBuf,
};

use anyhow::{
    Context,
    Result,
};

pub fn read_cargo_name(cargo_toml: &Path) -> Result<String> {
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

pub fn find_lib_deps(crate_dir: &Path) -> Result<Vec<(String, PathBuf)>> {
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

#[must_use]
pub fn derive_name(crate_name: &str) -> String {
    crate_name.replace('-', "_")
}
