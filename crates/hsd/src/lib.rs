use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// HSD/HSDX file format (RON).
///
/// Shared between author-time (`.hsdx`) and compiled client-ready (`.hsd`).
/// String refs in `scripts` and `assets` are relative paths:
/// - `.hsdx`: `./Cargo.toml`, `../dep/asset.hsdx`
/// - `.hsd`:  `./foo.wasm`,   `./dep.hsd`
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HsdFile {
    #[serde(default)]
    pub assets: BTreeMap<String, String>,
    #[serde(default)]
    pub nodes: BTreeMap<String, HsdNodeDef>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HsdNodeDef {
    #[serde(default)]
    pub scripts: Vec<String>,
}

impl HsdFile {
    pub fn parse(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
    }
}
