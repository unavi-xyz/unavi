use std::path::Path;

use anyhow::{
    Context,
    Result,
};
use blake3::Hash;

pub fn write_blob(out_dir: &Path, bytes: &[u8]) -> Result<Hash> {
    let hash = blake3::hash(bytes);
    let path = out_dir.join(hash.to_string());
    std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
    Ok(hash)
}
