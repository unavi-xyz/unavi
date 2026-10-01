//! Renders `shader/graph` (a compiled [`ShaderGraph`]) as a live material,
//! generating WGSL client-side from the validated graph.

pub mod cache;
pub mod codegen;
pub mod material;
pub(crate) mod systems;

use bevy::prelude::*;
use hsd::{
    attributes::shader::overrides::GraphOverridesAttr,
    property::{
        Payload,
        name::PropName,
    },
};

#[derive(Component, Clone)]
pub struct ShaderOverridesData(pub GraphOverridesAttr);

/// A prim's compiled graph. The hash is computed once at parse time so a
/// cache hit never has to decode the bytes.
#[derive(Component, Debug, Clone)]
pub struct ShaderGraphData {
    pub(crate) hash:  blake3::Hash,
    pub(crate) bytes: Vec<u8>,
}

/// Handles both the `graph` and `overrides` fields, dispatching on which one
/// was called.
pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    match name.field() {
        Some("graph") => match payload {
            Some(payload) => {
                commands.entity(prim).insert(ShaderGraphData {
                    hash:  blake3::hash(payload),
                    bytes: payload.to_vec(),
                });
            }
            None => {
                commands.entity(prim).remove::<ShaderGraphData>();
            }
        },
        Some("overrides") => match payload {
            Some(payload) => {
                commands
                    .entity(prim)
                    .insert(ShaderOverridesData(GraphOverridesAttr::decode(payload)?));
            }
            None => {
                commands.entity(prim).remove::<ShaderOverridesData>();
            }
        },
        _ => {}
    }
    Ok(())
}
