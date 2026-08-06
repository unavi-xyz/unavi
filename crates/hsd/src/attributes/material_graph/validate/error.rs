use thiserror::Error;

use crate::attributes::material_graph::{
    MAX_NODES,
    MAX_PUBLIC_INPUTS,
    MAX_TEXTURE_SAMPLES,
    node::Network,
    value::ValueKind,
};

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    #[error("{network:?} network has {count} nodes, exceeding the cap of {MAX_NODES}")]
    TooManyNodes { network: Network, count: usize },
    #[error("graph declares {0} public inputs, exceeding the cap of {MAX_PUBLIC_INPUTS}")]
    TooManyPublicInputs(usize),
    #[error("surface network samples {0} textures, exceeding the cap of {MAX_TEXTURE_SAMPLES}")]
    TooManyTextureSamples(usize),
    #[error("texture slot {0} is out of the {MAX_TEXTURE_SAMPLES}-slot range")]
    InvalidTextureSlot(u8),
    #[error("displacement network node {0} samples a texture, which is not supported in v1")]
    TextureSampleInDisplacement(usize),
    #[error("{network:?} network node {node} is not legal outside its own network")]
    WrongNetwork { network: Network, node: usize },
    #[error(
        "{network:?} network node {node} references node {target}, which is not at a strictly lower index"
    )]
    ForwardReference {
        network: Network,
        node:    usize,
        target:  u16,
    },
    #[error(
        "{network:?} network node {node} references public input {index}, which does not exist"
    )]
    UnknownInput {
        network: Network,
        node:    usize,
        index:   u16,
    },
    #[error("terminal {0} references node {1}, which does not exist")]
    UnknownTerminalNode(&'static str, u16),
    #[error("terminal {0} references public input {1}, which does not exist")]
    UnknownTerminalInput(&'static str, u16),
    #[error("{network:?} network node {node} port {port} expected {expected:?}, got {found:?}")]
    NodeTypeMismatch {
        network:  Network,
        node:     usize,
        port:     &'static str,
        expected: ValueKind,
        found:    ValueKind,
    },
    #[error("terminal {name} expected {expected:?}, got {found:?}")]
    TerminalTypeMismatch {
        name:     &'static str,
        expected: ValueKind,
        found:    ValueKind,
    },
    #[error("{network:?} network node {node} port {port} expected a vector kind, got {found:?}")]
    NotAVector {
        network: Network,
        node:    usize,
        port:    &'static str,
        found:   ValueKind,
    },
    #[error("{network:?} network node {node} extracts channel {channel} from a {kind:?}")]
    ChannelOutOfRange {
        network: Network,
        node:    usize,
        channel: u8,
        kind:    ValueKind,
    },
    #[error(
        "{network:?} network node {node} converts {from:?} to {to:?}; only vector kinds convert"
    )]
    InvalidConversion {
        network: Network,
        node:    usize,
        from:    ValueKind,
        to:      ValueKind,
    },
    #[error("{network:?} network node {node} holds a non-finite constant")]
    NonFiniteConst { network: Network, node: usize },
    #[error("public input {0} holds a non-finite value")]
    NonFinitePublicInput(usize),
    #[error("terminal {0} holds a non-finite constant")]
    NonFiniteTerminal(&'static str),
}
