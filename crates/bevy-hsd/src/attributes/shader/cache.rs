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
        validate::validate,
        value::GraphValue,
    },
    property::Payload,
};

use crate::{
    attributes::shader::{
        codegen,
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

/// Compiled graphs, keyed by the graph bytes' hash. Identical graphs across
/// documents compile once.
#[derive(Resource, Default)]
pub(crate) struct ShaderGraphCache {
    programs: HashMap<blake3::Hash, CachedGraph>,
    /// The graphs each document has charged against its cap. A program is
    /// dropped once no document still holds it.
    charged:  HashMap<Entity, HashSet<blake3::Hash>>,
}

impl ShaderGraphCache {
    pub(crate) fn get(&self, hash: blake3::Hash) -> Option<&CachedGraph> {
        self.programs.get(&hash)
    }

    /// The cached entry for `hash`, decoding, validating and compiling
    /// `bytes` only on a miss, and charging `doc`'s cap only for a build
    /// that succeeds.
    ///
    /// `None` when the graph is undecodable or invalid, or `doc` is at its
    /// cap of [`MAX_SHADER_PROGRAMS`] and `hash` is new to it.
    pub(crate) fn get_or_build(
        &mut self,
        hash: blake3::Hash,
        bytes: &[u8],
        doc: Entity,
        shaders: &mut Assets<Shader>,
    ) -> Option<CachedGraph> {
        if let Some(cached) = self.get(hash) {
            let cached = cached.clone();
            return self.charge(doc, hash).then_some(cached);
        }

        let graph = ShaderGraph::decode(bytes)
            .inspect_err(|err| warn!(?err, "undecodable shader graph"))
            .ok()?;
        let validated = validate(&graph)
            .inspect_err(|err| warn!(?err, "invalid shader graph"))
            .ok()?;
        let fragment_source = codegen::generate_fragment_shader(&graph, &validated)
            .inspect_err(|err| warn!(?err, "failed to generate fragment shader"))
            .ok()?;
        let vertex_source = codegen::generate_vertex_shader(&graph, &validated)
            .inspect_err(|err| warn!(?err, "failed to generate vertex shader"))
            .ok()?;

        let fragment = shaders.add(Shader::from_wgsl(
            fragment_source,
            format!("generated://shader/{hash}/fragment"),
        ));
        let vertex = vertex_source.map(|source| {
            shaders.add(Shader::from_wgsl(
                source,
                format!("generated://shader/{hash}/vertex"),
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
        self.programs.entry(hash).or_insert(built.clone());
        Some(built)
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
    let ShaderGraphCache { programs, charged } = &mut *cache;
    programs.retain(|hash, _| {
        !dropped.contains(hash) || charged.values().any(|held| held.contains(hash))
    });
}
