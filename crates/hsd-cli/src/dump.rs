//! `hsd-cli dump`: an inspection view of a compiled `.hsdz`.
//!
//! `.hsda`-shaped, but not source. Compilation replaced relative paths with
//! content, so this cannot be fed back to the compiler; it exists so a build
//! artifact is auditable without a hex editor.

use std::{
    collections::BTreeMap,
    path::Path,
};

use anyhow::{
    Context,
    Result,
};
use hsd::{
    attributes::{
        Attribute,
        collider::ColliderAttr,
        gravity_scale::GravityScaleAttr,
        image::ImageAttr,
        material::MaterialAttr,
        material_graph::GraphOverridesAttr,
        mesh::MeshAttr,
        name::NameAttr,
        portal::PortalAttr,
        rigid_body::RigidBodyAttr,
        spawn::SpawnAttr,
        xform::XformAttr,
    },
    id::PrimId,
    key,
    package::Package,
    property::{
        Parent,
        Property,
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
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    bulk:          BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children:      Vec<Self>,
}

#[derive(Default)]
struct Node {
    parent:        Option<Parent>,
    attributes:    BTreeMap<String, String>,
    relationships: BTreeMap<String, String>,
    bulk:          BTreeMap<String, String>,
}

pub fn dump_file(input: &Path) -> Result<String> {
    let bytes = std::fs::read(input).with_context(|| format!("reading {}", input.display()))?;
    let package =
        Package::decode(&bytes).with_context(|| format!("decoding {}", input.display()))?;

    let mut nodes: BTreeMap<PrimId, Node> = BTreeMap::new();
    for (raw, value) in &package.entries {
        match key::parse(raw) {
            Some(key::Key::Prop { prim, name }) if name == key::PARENT => {
                nodes.entry(prim).or_default().parent = Some(Parent::decode(value)?);
            }
            Some(key::Key::Prop { prim, name }) => {
                let node = nodes.entry(prim).or_default();
                match Property::decode(value)? {
                    Property::Relationship(target) => {
                        node.relationships
                            .insert(name.to_string(), target.to_string());
                    }
                    Property::Attribute(payload) => {
                        node.attributes
                            .insert(name.to_string(), render(&name, &payload));
                    }
                }
            }
            Some(key::Key::Bulk { prim, slot }) => {
                nodes
                    .entry(prim)
                    .or_default()
                    .bulk
                    .insert(slot.to_string(), format!("{} bytes", value.len()));
            }
            Some(key::Key::Meta) | None => {}
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
            bulk:          node.bulk.clone(),
            children:      build(nodes, Some(*id)),
        })
        .collect()
}

/// Renders a known attribute through the registry; an unknown one keeps its
/// key and its size, which is all a build that has never heard of it knows.
fn render(name: &str, payload: &[u8]) -> String {
    fn show<A: Attribute + std::fmt::Debug>(payload: &[u8]) -> String {
        A::decode(payload).map_or_else(|err| format!("<undecodable: {err}>"), |v| format!("{v:?}"))
    }

    match name {
        ColliderAttr::KEY => show::<ColliderAttr>(payload),
        GravityScaleAttr::KEY => show::<GravityScaleAttr>(payload),
        ImageAttr::KEY => show::<ImageAttr>(payload),
        MaterialAttr::KEY => show::<MaterialAttr>(payload),
        GraphOverridesAttr::KEY => show::<GraphOverridesAttr>(payload),
        MeshAttr::KEY => show::<MeshAttr>(payload),
        NameAttr::KEY => show::<NameAttr>(payload),
        PortalAttr::KEY => show::<PortalAttr>(payload),
        RigidBodyAttr::KEY => show::<RigidBodyAttr>(payload),
        SpawnAttr::KEY => show::<SpawnAttr>(payload),
        XformAttr::KEY => show::<XformAttr>(payload),
        _ => format!("<unknown, {} bytes>", payload.len()),
    }
}
