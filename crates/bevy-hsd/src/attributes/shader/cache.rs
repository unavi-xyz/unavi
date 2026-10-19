//! Compiled shader graphs, cached by content hash and charged against a
//! per-document program cap.

use bevy::{
    platform::collections::{
        HashMap,
        HashSet,
    },
    prelude::*,
    render::render_resource::Face,
};
use hsd::{
    attributes::shader::{
        ShaderGraph,
        node::Node,
        validate::validate,
        value::GraphValue,
    },
    property::Payload,
};

use crate::{
    attributes::shader::{
        codegen::{
            self,
            SEAL_NEUTRAL,
        },
        material::{
            alpha_mode,
            cull_mode,
            reads_scene,
            transmissive,
        },
    },
    document::Hsd,
};

/// A compiled graph's inputs to building the render material.
#[derive(Clone)]
pub(crate) struct CachedGraph {
    pub(crate) fragment:      Handle<Shader>,
    /// `None` when the graph has no
    /// [`hsd::attributes::shader::graph::DisplacementGraph`]. The mesh
    /// pipeline's own default vertex shader is used instead.
    pub(crate) vertex:        Option<Handle<Shader>>,
    pub(crate) public_inputs: Vec<GraphValue>,
    pub(crate) alpha_mode:    AlphaMode,
    pub(crate) cull_mode:     Option<Face>,
    pub(crate) cast_shadows:  bool,
    pub(crate) reads_scene:   bool,
    pub(crate) transmissive:  bool,
}

/// Distinct compiled graphs one document may hold at once.
///
/// Every graph edit changes the hash, so a document that varies one constant
/// would otherwise mint unbounded programs.
pub const MAX_SHADER_PROGRAMS: usize = 32;

/// One compiled program: the graph, and the seal colour it was generated for.
/// A graph with no `Node::Seal` always keys at [`SEAL_NEUTRAL`], so it is
/// never duplicated per document.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProgramKey {
    hash: blake3::Hash,
    seal: [u32; 4],
}

fn seal_bits(seal: [f32; 4]) -> [u32; 4] {
    seal.map(f32::to_bits)
}

fn hex_bits(seal: [f32; 4]) -> String {
    seal_bits(seal).map(|bits| format!("{bits:08x}")).concat()
}

fn uses_seal(graph: &ShaderGraph) -> bool {
    graph
        .surface
        .nodes
        .iter()
        .chain(graph.displacement.iter().flat_map(|d| &d.nodes))
        .any(|node| matches!(node, Node::Seal))
}

/// Compiled graphs, keyed by the graph bytes' hash and resolved seal.
/// Identical graphs across documents compile once.
#[derive(Resource, Default)]
pub(crate) struct ShaderGraphCache {
    programs:  HashMap<ProgramKey, CachedGraph>,
    /// Graphs known to contain no `Node::Seal`.
    seal_free: HashSet<blake3::Hash>,
    /// The graphs each document has charged against its cap. A program is
    /// dropped once no document still holds it.
    charged:   HashMap<Entity, HashSet<blake3::Hash>>,
}

impl ShaderGraphCache {
    /// The key `hash` compiles under for a document whose seal resolves to
    /// `seal`.
    pub(crate) fn key(&self, hash: blake3::Hash, seal: [f32; 4]) -> ProgramKey {
        let seal = if self.seal_free.contains(&hash) {
            SEAL_NEUTRAL
        } else {
            seal
        };
        ProgramKey {
            hash,
            seal: seal_bits(seal),
        }
    }

    /// Whether `hash` is a built graph with no `Node::Seal`, which no seal
    /// change can affect.
    pub(crate) fn is_seal_free(&self, hash: blake3::Hash) -> bool {
        self.seal_free.contains(&hash)
    }

    pub(crate) fn get(&self, key: ProgramKey) -> Option<&CachedGraph> {
        self.programs.get(&key)
    }

    /// The cached entry for `hash` under `seal` and the key it is held
    /// under, decoding, validating and compiling `bytes` only on a miss, and
    /// charging `doc`'s cap only for a build that succeeds.
    ///
    /// `None` when the graph is undecodable or invalid, or `doc` is at its
    /// cap of [`MAX_SHADER_PROGRAMS`] and `hash` is new to it.
    pub(crate) fn get_or_build(
        &mut self,
        hash: blake3::Hash,
        bytes: &[u8],
        seal: [f32; 4],
        doc: Entity,
        shaders: &mut Assets<Shader>,
    ) -> Option<(ProgramKey, CachedGraph)> {
        let key = self.key(hash, seal);
        if let Some(cached) = self.get(key) {
            let cached = cached.clone();
            return self.charge(doc, hash).then_some((key, cached));
        }

        let graph = ShaderGraph::decode(bytes)
            .inspect_err(|err| warn!(?err, "undecodable shader graph"))
            .ok()?;
        let validated = validate(&graph)
            .inspect_err(|err| warn!(?err, "invalid shader graph"))
            .ok()?;
        let sealed = uses_seal(&graph);
        let seal = if sealed { seal } else { SEAL_NEUTRAL };
        let fragment_source = codegen::generate_fragment_shader(&graph, &validated, seal)
            .inspect_err(|err| warn!(?err, "failed to generate fragment shader"))
            .ok()?;
        let vertex_source = codegen::generate_vertex_shader(&graph, &validated, seal)
            .inspect_err(|err| warn!(?err, "failed to generate vertex shader"))
            .ok()?;

        let fragment = shaders.add(Shader::from_wgsl(
            fragment_source,
            format!("generated://shader/{hash}/{}/fragment", hex_bits(seal)),
        ));
        let vertex = vertex_source.map(|source| {
            shaders.add(Shader::from_wgsl(
                source,
                format!("generated://shader/{hash}/{}/vertex", hex_bits(seal)),
            ))
        });

        let built = CachedGraph {
            fragment,
            vertex,
            public_inputs: graph.public_inputs.clone(),
            alpha_mode: alpha_mode(graph.surface.blend),
            cull_mode: cull_mode(graph.surface.cull),
            cast_shadows: graph.surface.cast_shadows,
            reads_scene: reads_scene(&graph),
            transmissive: transmissive(&graph),
        };

        if !self.charge(doc, hash) {
            warn!(
                "document is at its cap of {MAX_SHADER_PROGRAMS} shader programs; ignoring another"
            );
            return None;
        }
        if !sealed {
            self.seal_free.insert(hash);
        }
        let key = self.key(hash, seal);
        self.programs.entry(key).or_insert(built.clone());
        Some((key, built))
    }

    /// Charges `hash` against `doc`'s cap if it is not already charged.
    /// `false` once `doc` is at [`MAX_SHADER_PROGRAMS`] distinct graphs and
    /// `hash` is not already one of them.
    fn charge(&mut self, doc: Entity, hash: blake3::Hash) -> bool {
        let charged = self.charged.entry(doc).or_default();
        if charged.contains(&hash) {
            return true;
        }
        if charged.len() >= MAX_SHADER_PROGRAMS {
            return false;
        }
        charged.insert(hash);
        true
    }
}

pub(crate) fn evict_document_shaders(
    trigger: On<Remove, Hsd>,
    mut cache: ResMut<ShaderGraphCache>,
) {
    let Some(dropped) = cache.charged.remove(&trigger.entity) else {
        return;
    };
    let ShaderGraphCache {
        programs, charged, ..
    } = &mut *cache;
    programs.retain(|key, _| {
        !dropped.contains(&key.hash) || charged.values().any(|held| held.contains(&key.hash))
    });
}
