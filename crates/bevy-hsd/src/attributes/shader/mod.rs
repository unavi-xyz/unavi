//! Renders `shader/graph` (a compiled [`ShaderGraph`]) as a live material,
//! generating WGSL locally from the validated graph.

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

/// The local user's seal colour, which `Node::Seal` draws in a [`Sealed`]
/// document. Only the app that owns the device sets it, and no script can
/// read it.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ShellSeal(pub LinearRgba);

impl Default for ShellSeal {
    fn default() -> Self {
        let [r, g, b, a] = codegen::SEAL_NEUTRAL;
        Self(LinearRgba::new(r, g, b, a))
    }
}

/// Marks a document entity the local user authors, so `Node::Seal` draws
/// [`ShellSeal`] in its prims and grey everywhere else.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Sealed;

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
