//! `hsd-cli dump`: an inspection view of a compiled `.hsdz`.
//!
//! `.hsda`-shaped, but not source. Compilation replaced relative paths with
//! content, so this cannot be fed back to the compiler.

use std::{
    collections::BTreeMap,
    path::Path,
};

use anyhow::{
    Context,
    Result,
};
use hsd::{
    format::package::Package,
    id::PrimId,
    key,
    property::{
        Payload,
        Property,
        name::PropName,
        render_bytes,
        value::Value,
    },
    schema::{
        collider::{
            ColliderIndices,
            ColliderKind,
            ColliderVertices,
        },
        gravity_scale::GravityScaleAttr,
        image::{
            ImageData,
            ImageSampler,
        },
        material::MaterialAttr,
        mesh::{
            self,
            MeshIndices,
            MeshStream,
            Topology,
        },
        name::NameAttr,
        parent::ParentAttr,
        portal::PortalAttr,
        reference::{
            self,
            LayerKey,
            ReferenceAttr,
        },
        rigid_body::RigidBodyAttr,
        script::ScriptAttr,
        shader::{
            ShaderGraph,
            overrides::GraphOverridesAttr,
        },
        spawn::SpawnAttr,
        text::TextAttr,
        xform::XformAttr,
    },
};
use ron::extensions::Extensions;
use serde::Serialize;

#[derive(Serialize, Default)]
struct DumpPrim {
    id:            String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    attributes:    BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    relationships: BTreeMap<String, String>,
    /// What this prim says about the prims of the document it references,
    /// keyed by target prim.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    overrides:     BTreeMap<String, BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children:      Vec<Self>,
}

#[derive(Default)]
struct Node {
    parent:        Option<ParentAttr>,
    attributes:    BTreeMap<String, String>,
    relationships: BTreeMap<String, String>,
    overrides:     BTreeMap<String, BTreeMap<String, String>>,
}

pub fn dump_file(input: &Path) -> Result<String> {
    let bytes = std::fs::read(input).with_context(|| format!("reading {}", input.display()))?;
    let package =
        Package::decode(&bytes).with_context(|| format!("decoding {}", input.display()))?;

    let mut nodes: BTreeMap<PrimId, Node> = BTreeMap::new();
    for (raw, value) in &package.entries {
        match key::Key::parse(raw) {
            Some(key::Key::Prop { prim, name }) if name == ParentAttr::NAME => {
                nodes.entry(prim).or_default().parent = ParentAttr::from_wire(value)?;
            }
            Some(key::Key::Prop { prim, name }) => {
                let node = nodes.entry(prim).or_default();
                match Value::decode(value)? {
                    Value::Relationship(target) => {
                        node.relationships
                            .insert(name.to_string(), target.to_string());
                    }
                    Value::Attribute(payload) => {
                        node.attributes
                            .insert(name.to_string(), render(&name, &payload));
                    }
                }
            }
            Some(key::Key::Nested { prim, group, tail }) if group == reference::GROUP => {
                if let Some(LayerKey { target, name }) = LayerKey::parse(&tail) {
                    nodes
                        .entry(prim)
                        .or_default()
                        .overrides
                        .entry(target.to_string())
                        .or_default()
                        .insert(name.to_string(), render_override(&name, value)?);
                }
            }
            Some(key::Key::Meta | key::Key::Nested { .. }) | None => {}
        }
    }

    let roots = build(&nodes, None);
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .to_string_pretty(&roots, ron::ser::PrettyConfig::default())
        .context("serializing dump")
}

fn build(nodes: &BTreeMap<PrimId, Node>, parent: Option<PrimId>) -> Vec<DumpPrim> {
    nodes
        .iter()
        .filter(|(_, node)| node.parent.and_then(|p| p.prim()) == parent)
        .filter(|(_, node)| node.parent.is_some())
        .map(|(id, node)| DumpPrim {
            id:            id.to_string(),
            attributes:    node.attributes.clone(),
            relationships: node.relationships.clone(),
            overrides:     node.overrides.clone(),
            children:      build(nodes, Some(*id)),
        })
        .collect()
}

/// Renders an override's value, which carries whichever kind of key it names.
/// An empty value blocks the key rather than stating one.
fn render_override(name: &PropName, value: &[u8]) -> Result<String> {
    if value.is_empty() {
        return Ok("<blocked>".to_owned());
    }
    if *name == ParentAttr::NAME {
        return Ok(format!("{:?}", ParentAttr::from_wire(value)?));
    }
    Ok(match Value::decode(value)? {
        Value::Relationship(target) => target.to_string(),
        Value::Attribute(payload) => render(name, &payload),
    })
}

/// The attributes dump decodes. Any other renders as its size.
const DECODED: &[(PropName, fn(&[u8]) -> String)] = &[
    (ColliderKind::NAME, ColliderKind::render),
    (ColliderVertices::NAME, ColliderVertices::render),
    (ColliderIndices::NAME, ColliderIndices::render),
    (GravityScaleAttr::NAME, GravityScaleAttr::render),
    (ImageSampler::NAME, ImageSampler::render),
    (ImageData::NAME, ImageData::render),
    (MaterialAttr::NAME, MaterialAttr::render),
    (Topology::NAME, Topology::render),
    (MeshIndices::NAME, MeshIndices::render),
    (NameAttr::NAME, NameAttr::render),
    (PortalAttr::NAME, PortalAttr::render),
    (ReferenceAttr::NAME, ReferenceAttr::render),
    (RigidBodyAttr::NAME, RigidBodyAttr::render),
    (ScriptAttr::NAME, ScriptAttr::render),
    (ShaderGraph::NAME, ShaderGraph::render),
    (GraphOverridesAttr::NAME, GraphOverridesAttr::render),
    (SpawnAttr::NAME, SpawnAttr::render),
    (TextAttr::NAME, TextAttr::render),
    (XformAttr::NAME, XformAttr::render),
];

/// A mesh stream, named by a prefix, is rendered from its own type.
fn render(name: &PropName, payload: &[u8]) -> String {
    if mesh::stream_of(name).is_some() {
        return MeshStream::decode(payload).map_or_else(
            |err| format!("<undecodable: {err}>"),
            |value| render_bytes(value.0.len()),
        );
    }
    DECODED.iter().find(|(known, _)| known == name).map_or_else(
        || format!("<unknown, {} bytes>", payload.len()),
        |(_, render)| render(payload),
    )
}
