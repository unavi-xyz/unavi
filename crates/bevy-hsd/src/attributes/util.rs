use loro::{ContainerID, Index, event::Diff};

use crate::attributes::ParseError;

/// Parses the top-level updated keys out of a diff map.
pub fn shallow_map_updated_keys(
    path: &[(ContainerID, Index)],
    diff: Diff,
) -> Result<Vec<String>, ParseError> {
    let keys = if path.is_empty() {
        diff.into_map()
            .map_err(|_| anyhow::anyhow!("invalid diff type"))?
            .updated
            .into_keys()
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![
            path[0]
                .1
                .as_key()
                .ok_or_else(|| anyhow::anyhow!("invalid index type"))?
                .to_string(),
        ]
    };
    Ok(keys)
}
