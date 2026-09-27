use std::collections::BTreeMap;

use const_format::concatcp;
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::{
    prop_name,
    property::{
        Property,
        name::PropName,
    },
    schema::shader::{
        GROUP,
        ShaderGraph,
        value::{
            GraphValue,
            ValueKind,
            is_finite,
        },
    },
};

/// A prim's per-instance public-input override for its compiled graph.
///
/// Never carries the graph itself, which is [`ShaderGraph`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphOverridesAttr {
    /// Public-input index -> override value. Empty if the graph's own
    /// defaults (`ShaderGraph::public_inputs`) are used as-is.
    pub overrides: BTreeMap<u16, GraphValue>,
}

impl Property for GraphOverridesAttr {
    const NAME: PropName = prop_name!(concatcp!(GROUP, "/overrides"));
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
    #[error("override for public input {0} is non-finite")]
    NonFinite(u16),
}

/// Cross-checks overrides against the graph they apply to.
///
/// The two attributes can arrive or go stale independently, so call this
/// whenever either changes.
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
        if !is_finite(*value) {
            return Err(OverridesError::NonFinite(index));
        }
    }
    Ok(())
}
