//! `wired:shading`: shader graphs a script builds as data.

use std::collections::BTreeMap;

use hsd::{
    attributes::shader::{
        MAX_PUBLIC_INPUTS,
        ShaderGraph,
        overrides::GraphOverridesAttr,
        validate::validate,
        value::{
            GraphValue,
            is_finite,
        },
    },
    id::PrimId,
    property::{
        Payload,
        Property,
    },
};
use unavi_policy::quota::Flow;

use crate::{
    error::ScriptError,
    host::{
        ScriptHost,
        scene::edit::{
            Layer,
            write_fields,
        },
    },
};

/// Draws `prim` with `graph`, or stops on `None`, dropping its overrides too.
///
/// A graph is bounded by its shape, so building one is no more dangerous than
/// building a mesh; validation is what enforces the shape.
pub async fn set_graph(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
    layer: Layer,
    graph: Option<ShaderGraph>,
) -> Result<(), ScriptError> {
    let doc = host.owned_document(doc)?.clone();
    let fields = match graph {
        Some(graph) => {
            validate(&graph).map_err(|err| ScriptError::invalid(err.to_string()))?;
            crate::quota::take(&host.quota, Flow::BlobUpload, 1)?;
            vec![(
                ShaderGraph::NAME,
                Some(graph.encode().map_err(ScriptError::internal)?),
            )]
        }
        None => vec![(ShaderGraph::NAME, None), (GraphOverridesAttr::NAME, None)],
    };
    write_fields(host, &doc, layer, prim, fields).await
}

pub fn overrides(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
) -> Result<Vec<(u16, GraphValue)>, ScriptError> {
    host.document(doc)?.read(|state| {
        state
            .attribute::<GraphOverridesAttr>(prim)
            .and_then(Result::ok)
            .map_or_default(|attr| attr.overrides.into_iter().collect())
    })
}

/// Replaces every override of `prim`'s graph in `layer`. An empty set clears
/// them rather than storing an empty map.
pub async fn set_overrides(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
    layer: Layer,
    values: Vec<(u16, GraphValue)>,
) -> Result<(), ScriptError> {
    let doc = host.owned_document(doc)?.clone();
    if values.len() > MAX_PUBLIC_INPUTS {
        return Err(ScriptError::invalid("a graph has at most 16 public inputs"));
    }
    if !values.iter().all(|(_, value)| is_finite(*value)) {
        return Err(ScriptError::invalid("an override must be finite"));
    }
    let overrides = values.into_iter().collect::<BTreeMap<_, _>>();
    let value = if overrides.is_empty() {
        None
    } else {
        Some(
            GraphOverridesAttr { overrides }
                .encode()
                .map_err(ScriptError::internal)?,
        )
    };
    write_fields(
        host,
        &doc,
        layer,
        prim,
        vec![(GraphOverridesAttr::NAME, value)],
    )
    .await
}
