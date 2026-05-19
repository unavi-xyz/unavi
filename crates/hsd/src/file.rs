use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use loro::{LoroDoc, TreeID, TreeParentId};
use lorosurgeon::{ByteArray, MaybeMissing, Reconcile, reconcile::RootReconciler};
use ron::extensions::Extensions;
use serde::{Deserialize, Serialize};

use crate::{
    HSD_CONTAINER_ID, PrimMeta,
    attributes::{
        Attributes,
        asset::AssetAttr,
        collider::ColliderAttr,
        image::ImageAttr,
        material::{ColorVec, MaterialAttr},
        mesh::{MeshAttr, Topology},
        name::NameAttr,
        rigid_body::{RigidBodyAttr, RigidBodyKind},
        script::ScriptAttr,
        xform::XformAttr,
    },
};

fn ron_options() -> ron::Options {
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
}

/// Compiled `.hsd` file — a list of root-level prims. Each prim may have
/// children forming a tree. Blob ids are `[u8; 32]` (blake3 hashes).
#[derive(Serialize, Deserialize, Default)]
pub struct HsdFile(pub Vec<HsdFilePrim>);

impl HsdFile {
    pub fn from_ron(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron_options().from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron_options().to_string_pretty(self, ron::ser::PrettyConfig::default())
    }

    /// Walk the prim tree and populate `doc` via lorosurgeon reconciliation.
    pub fn load_into_doc(&self, doc: &LoroDoc) -> Result<()> {
        let tree = doc.get_tree(&*HSD_CONTAINER_ID);
        let mut id_map: HashMap<String, TreeID> = HashMap::new();
        let mut pairs: Vec<(&HsdFilePrim, TreeID)> = Vec::new();

        create_prims(&tree, TreeParentId::Root, &self.0, &mut id_map, &mut pairs);

        for (prim, tree_id) in &pairs {
            let meta = tree.get_meta(*tree_id)?;

            let mut rels: BTreeMap<String, String> = BTreeMap::new();
            for (key, val) in &prim.relationships {
                let target = id_map
                    .get(val)
                    .map_or_else(|| val.clone(), |tid| tid.to_string());
                rels.insert(key.clone(), target);
            }

            let prim_meta = PrimMeta {
                attributes: MaybeMissing::Present(attrs_from_file(&prim.attributes)),
                relationships: if rels.is_empty() {
                    MaybeMissing::Missing
                } else {
                    MaybeMissing::Present(rels)
                },
            };

            prim_meta
                .reconcile(RootReconciler::new(meta))
                .map_err(|e| anyhow::anyhow!("{e}"))?;
        }

        doc.commit();
        Ok(())
    }
}

fn create_prims<'a>(
    tree: &loro::LoroTree,
    parent: TreeParentId,
    prims: &'a [HsdFilePrim],
    id_map: &mut HashMap<String, TreeID>,
    out: &mut Vec<(&'a HsdFilePrim, TreeID)>,
) {
    for prim in prims {
        let tree_id = tree.create(parent).expect("create prim");
        if let Some(name) = &prim.attributes.name {
            id_map.insert(name.clone(), tree_id);
        }
        out.push((prim, tree_id));
        create_prims(tree, TreeParentId::Node(tree_id), &prim.children, id_map, out);
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct HsdFilePrim {
    #[serde(default)]
    pub attributes: HsdFileAttributes,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub relationships: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<HsdFilePrim>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdFileAttributes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<[u8; 32]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collider: Option<HsdFileCollider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<HsdFileImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<HsdFileMaterial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<HsdFileMesh>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rigid_body: Option<HsdFileRigidBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<[u8; 32]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xform: Option<HsdFileXform>,
}

#[derive(Serialize, Deserialize)]
pub enum HsdFileCollider {
    Capsule { height: f64, radius: f64 },
    ConvexHull([u8; 32]),
    Cuboid { x: f64, y: f64, z: f64 },
    Cylinder { height: f64, radius: f64 },
    Sphere(f64),
    Trimesh { indices: [u8; 32], vertices: [u8; 32] },
}

#[derive(Serialize, Deserialize, Default)]
pub struct HsdFileImage {
    pub data: [u8; 32],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address_mode_u: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address_mode_v: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address_mode_w: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mag_filter: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_filter: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mipmap_filter: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub srgb: Option<bool>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct HsdFileMaterial {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alpha_cutoff: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alpha_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_color: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_color_texture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub double_sided: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emissive: Option<Vec<f64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emissive_texture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metallic: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metallic_roughness_texture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normal_texture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occlusion_texture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roughness: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct HsdFileMesh {
    pub topology: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, [u8; 32]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indices: Option<[u8; 32]>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct HsdFileRigidBody {
    #[serde(default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angular_damping: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub friction: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linear_damping: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mass: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restitution: Option<f64>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct HsdFileXform {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Vec<f32>>,
}

fn opt_to_maybe<T>(opt: Option<T>) -> MaybeMissing<T> {
    match opt {
        Some(x) => MaybeMissing::Present(x),
        None => MaybeMissing::Missing,
    }
}

fn attrs_from_file(a: &HsdFileAttributes) -> Attributes {
    Attributes {
        asset: opt_to_maybe(a.asset.map(|b| AssetAttr(ByteArray::new(b)))),
        collider: opt_to_maybe(a.collider.as_ref().map(collider_from_file)),
        image: opt_to_maybe(a.image.as_ref().map(image_from_file)),
        material: opt_to_maybe(a.material.as_ref().map(material_from_file)),
        mesh: opt_to_maybe(a.mesh.as_ref().map(mesh_from_file)),
        name: opt_to_maybe(a.name.as_deref().map(|s| NameAttr(s.to_owned()))),
        rigid_body: opt_to_maybe(a.rigid_body.as_ref().map(rigid_body_from_file)),
        script: opt_to_maybe(a.script.map(|b| ScriptAttr(ByteArray::new(b)))),
        xform: opt_to_maybe(a.xform.as_ref().map(xform_from_file)),
    }
}

fn collider_from_file(c: &HsdFileCollider) -> ColliderAttr {
    match c {
        HsdFileCollider::Capsule { height, radius } => {
            ColliderAttr::Capsule { height: *height, radius: *radius }
        }
        HsdFileCollider::ConvexHull(b) => ColliderAttr::ConvexHull(ByteArray::new(*b)),
        HsdFileCollider::Cuboid { x, y, z } => ColliderAttr::Cuboid { x: *x, y: *y, z: *z },
        HsdFileCollider::Cylinder { height, radius } => {
            ColliderAttr::Cylinder { height: *height, radius: *radius }
        }
        HsdFileCollider::Sphere(r) => ColliderAttr::Sphere(*r),
        HsdFileCollider::Trimesh { indices, vertices } => ColliderAttr::Trimesh {
            indices: ByteArray::new(*indices),
            vertices: ByteArray::new(*vertices),
        },
    }
}

fn image_from_file(img: &HsdFileImage) -> ImageAttr {
    ImageAttr {
        data: ByteArray::new(img.data),
        address_mode_u: opt_to_maybe(img.address_mode_u),
        address_mode_v: opt_to_maybe(img.address_mode_v),
        address_mode_w: opt_to_maybe(img.address_mode_w),
        mag_filter: opt_to_maybe(img.mag_filter),
        min_filter: opt_to_maybe(img.min_filter),
        mipmap_filter: opt_to_maybe(img.mipmap_filter),
        srgb: opt_to_maybe(img.srgb),
    }
}

fn material_from_file(m: &HsdFileMaterial) -> MaterialAttr {
    MaterialAttr {
        alpha_cutoff: opt_to_maybe(m.alpha_cutoff),
        alpha_mode: opt_to_maybe(m.alpha_mode.clone()),
        base_color: opt_to_maybe(m.base_color.clone().map(ColorVec)),
        base_color_texture: opt_to_maybe(m.base_color_texture.clone()),
        double_sided: opt_to_maybe(m.double_sided),
        emissive: opt_to_maybe(m.emissive.clone().map(ColorVec)),
        emissive_texture: opt_to_maybe(m.emissive_texture.clone()),
        metallic: opt_to_maybe(m.metallic),
        metallic_roughness_texture: opt_to_maybe(m.metallic_roughness_texture.clone()),
        normal_texture: opt_to_maybe(m.normal_texture.clone()),
        occlusion_texture: opt_to_maybe(m.occlusion_texture.clone()),
        roughness: opt_to_maybe(m.roughness),
    }
}

fn mesh_from_file(mesh: &HsdFileMesh) -> MeshAttr {
    let topology = match mesh.topology.as_str() {
        "LineList" => Topology::LineList,
        "LineStrip" => Topology::LineStrip,
        "PointList" => Topology::PointList,
        "TriangleStrip" => Topology::TriangleStrip,
        _ => Topology::TriangleList,
    };
    MeshAttr {
        topology,
        attributes: mesh
            .attributes
            .iter()
            .map(|(k, v)| (k.clone(), ByteArray::new(*v)))
            .collect(),
        indices: opt_to_maybe(mesh.indices.map(|b| ByteArray::new(b))),
    }
}

fn rigid_body_from_file(rb: &HsdFileRigidBody) -> RigidBodyAttr {
    let kind = match rb.kind.as_str() {
        "Static" => RigidBodyKind::Static,
        "Kinematic" => RigidBodyKind::Kinematic,
        _ => RigidBodyKind::Dynamic,
    };
    RigidBodyAttr {
        kind,
        angular_damping: opt_to_maybe(rb.angular_damping),
        friction: opt_to_maybe(rb.friction),
        linear_damping: opt_to_maybe(rb.linear_damping),
        mass: opt_to_maybe(rb.mass),
        restitution: opt_to_maybe(rb.restitution),
    }
}

fn xform_from_file(x: &HsdFileXform) -> XformAttr {
    XformAttr {
        translation: x.translation.clone().unwrap_or_default(),
        rotation: x.rotation.clone().unwrap_or_else(|| vec![0.0, 0.0, 0.0, 1.0]),
        scale: x.scale.clone().unwrap_or_else(|| vec![1.0, 1.0, 1.0]),
    }
}
