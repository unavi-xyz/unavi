use std::collections::BTreeMap;

use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::attributes::{
    Attribute,
    material_graph::{
        GraphValue,
        ShaderGraph,
        ValueKind,
    },
};

/// The small attribute pairing a graph's per-instance public-input tint.
///
/// Follows `MaterialX`'s "bind a nodegraph, override its public inputs"
/// pattern. Never carries the graph itself — that is `material:graph_data`,
/// slot content, since a hash may not appear inside an attribute payload.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphOverridesAttr {
    /// Public-input index -> override value. Empty if the graph's own
    /// defaults (`ShaderGraph::public_inputs`) are used as-is.
    pub overrides: BTreeMap<u16, GraphValue>,
}

impl Attribute for GraphOverridesAttr {
    const KEY: &'static str = "material:graph";
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum OverridesError {
    #[error("override targets public input {0}, which the graph does not declare")]
    UnknownInput(u16),
    #[error("override for public input {index} expected {expected:?}, got {found:?}")]
    TypeMismatch {
        index:    u16,
        expected: ValueKind,
        found:    ValueKind,
    },
}

/// Cross-checks overrides against the graph they apply to.
///
/// The two are separate entries (an attribute and a slot) that can
/// arrive out of order or go stale independently, so this is re-run whenever
/// either changes, not folded into [`super::validate::validate`].
pub fn validate_overrides(
    graph: &ShaderGraph,
    overrides: &GraphOverridesAttr,
) -> Result<(), OverridesError> {
    for (&index, value) in &overrides.overrides {
        let expected = graph
            .public_inputs
            .get(usize::from(index))
            .ok_or(OverridesError::UnknownInput(index))?
            .kind();
        if value.kind() != expected {
            return Err(OverridesError::TypeMismatch {
                index,
                expected,
                found: value.kind(),
            });
        }
    }
    Ok(())
}
