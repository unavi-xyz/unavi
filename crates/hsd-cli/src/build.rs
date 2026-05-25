use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    path::{
        Path,
        PathBuf,
    },
};

use anyhow::{
    Context,
    Result,
    bail,
};
use blake3::Hash;
use hsd::{
    attributes::{
        Attributes,
        asset::AssetAttr,
        collider::ColliderAttr,
        image::ImageAttr,
        material::{
            ColorVec,
            MaterialAttr,
        },
        name::NameAttr,
        rigid_body::{
            RigidBodyAttr,
            RigidBodyKind,
        },
        script::ScriptAttr,
        xform::XformAttr,
    },
    file::{
        HsdFile,
        HsdFilePrim,
    },
};
use loro_surgeon::bytes::ByteArray;
use ron::extensions::Extensions;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    blobs::write_blob,
    wasm::build_wasm_for_crate,
};

pub struct BuildOutput {
    pub hash: Hash,
}

fn ron_options() -> ron::Options {
    ron::Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
}

#[derive(Serialize, Deserialize, Default)]
pub struct Hsdx(pub Vec<HsdxPrim>);

impl Hsdx {
    pub fn parse(s: &str) -> Result<Self, ron::error::SpannedError> {
        ron_options().from_str(s)
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron_options().to_string_pretty(self, ron::ser::PrettyConfig::default())
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdxPrim {
    pub attributes:    HsdxAttributes,
    pub relationships: BTreeMap<String, String>,
    pub children:      Vec<Self>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdxAttributes {
    pub asset:      Option<String>,
    pub collider:   Option<HsdxCollider>,
    pub image:      Option<HsdxImage>,
    pub material:   Option<HsdxMaterial>,
    pub name:       Option<String>,
    pub rigid_body: Option<HsdxRigidBody>,
    pub script:     Option<String>,
    pub xform:      Option<HsdxXform>,
}

#[derive(Serialize, Deserialize)]
pub enum HsdxCollider {
    Capsule { height: f64, radius: f64 },
    ConvexHull(String),
    Cuboid { x: f64, y: f64, z: f64 },
    Cylinder { height: f64, radius: f64 },
    Sphere(f64),
    Trimesh { indices: String, vertices: String },
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdxImage {
    pub data:           String,
    pub address_mode_u: Option<i64>,
    pub address_mode_v: Option<i64>,
    pub address_mode_w: Option<i64>,
    pub mag_filter:     Option<i64>,
    pub min_filter:     Option<i64>,
    pub mipmap_filter:  Option<i64>,
    pub srgb:           Option<bool>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdxMaterial {
    pub alpha_cutoff:               Option<f64>,
    pub alpha_mode:                 Option<String>,
    pub base_color:                 Option<Vec<f64>>,
    pub base_color_texture:         Option<String>,
    pub double_sided:               Option<bool>,
    pub emissive:                   Option<Vec<f64>>,
    pub emissive_texture:           Option<String>,
    pub metallic:                   Option<f64>,
    pub metallic_roughness_texture: Option<String>,
    pub normal_texture:             Option<String>,
    pub occlusion_texture:          Option<String>,
    pub roughness:                  Option<f64>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdxRigidBody {
    pub kind:            String,
    pub angular_damping: Option<f64>,
    pub friction:        Option<f64>,
    pub linear_damping:  Option<f64>,
    pub mass:            Option<f64>,
    pub restitution:     Option<f64>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct HsdxXform {
    pub translation: Option<Vec<f32>>,
    pub rotation:    Option<Vec<f32>>,
    pub scale:       Option<Vec<f32>>,
}

pub fn build_hsdx_to_hsd<S: std::hash::BuildHasher>(
    input: &Path,
    out_dir: &Path,
    built: &mut HashMap<String, Hash, S>,
) -> Result<BuildOutput> {
    let input_abs =
        std::fs::canonicalize(input).with_context(|| format!("resolving {}", input.display()))?;
    let input_dir = input_abs.parent().context("input has no parent dir")?;

    let src = std::fs::read_to_string(&input_abs)
        .with_context(|| format!("reading {}", input_abs.display()))?;
    let hsdx = Hsdx::parse(&src).with_context(|| format!("parsing {}", input_abs.display()))?;

    let mut file_prims = Vec::new();
    for prim in &hsdx.0 {
        file_prims.push(compile_prim(prim, input_dir, out_dir, built)?);
    }

    let hsd_file = HsdFile(file_prims);
    let bytes = hsd_file.to_ron()?.into_bytes();
    let hash = write_blob(out_dir, &bytes)?;

    let crate_dir_name = input_dir
        .file_name()
        .with_context(|| format!("input dir has no name: {}", input_dir.display()))?
        .to_string_lossy();
    let output_name = crate_dir_name.replace('-', "_");
    let out_hsd = out_dir.join(format!("{output_name}.hsd"));
    std::fs::write(&out_hsd, &bytes).with_context(|| format!("writing {}", out_hsd.display()))?;
    println!("wrote {}", out_hsd.display());

    Ok(BuildOutput { hash })
}

fn compile_prim<S: std::hash::BuildHasher>(
    prim: &HsdxPrim,
    input_dir: &Path,
    out_dir: &Path,
    built: &mut HashMap<String, Hash, S>,
) -> Result<HsdFilePrim> {
    let attrs = compile_attrs(&prim.attributes, input_dir, out_dir, built)?;

    let mut children = Vec::new();
    for child in &prim.children {
        children.push(compile_prim(child, input_dir, out_dir, built)?);
    }

    Ok(HsdFilePrim {
        attributes: attrs,
        relationships: prim.relationships.clone(),
        children,
    })
}

fn compile_attrs<S: std::hash::BuildHasher>(
    attrs: &HsdxAttributes,
    input_dir: &Path,
    out_dir: &Path,
    built: &mut HashMap<String, Hash, S>,
) -> Result<Attributes> {
    let asset = attrs
        .asset
        .as_deref()
        .map(|rel| compile_asset(rel, input_dir, out_dir, built))
        .transpose()?;

    let script = attrs
        .script
        .as_deref()
        .map(|rel| compile_script(rel, input_dir, out_dir, built))
        .transpose()?;

    let image = attrs
        .image
        .as_ref()
        .map(|img| compile_image(img, input_dir, out_dir))
        .transpose()?;

    Ok(Attributes {
        asset:      (asset.map(|h| AssetAttr(ByteArray::new(h)))),
        collider:   (attrs.collider.as_ref().map(compile_collider).transpose()?),
        image:      (image),
        material:   (attrs.material.as_ref().map(compile_material)),
        mesh:       None,
        name:       (attrs.name.clone().map(NameAttr)),
        rigid_body: (attrs
            .rigid_body
            .as_ref()
            .map(compile_rigid_body)
            .transpose()?),
        script:     (script.map(|h| ScriptAttr(ByteArray::new(h)))),
        xform:      (attrs.xform.as_ref().map(compile_xform)),
    })
}

fn compile_asset<S: std::hash::BuildHasher>(
    rel: &str,
    input_dir: &Path,
    out_dir: &Path,
    built: &mut HashMap<String, Hash, S>,
) -> Result<[u8; 32]> {
    let dep_path = input_dir.join(rel);
    let output = build_hsdx_to_hsd(&dep_path, out_dir, built)?;
    Ok(*output.hash.as_bytes())
}

fn compile_script<S: std::hash::BuildHasher>(
    rel: &str,
    input_dir: &Path,
    out_dir: &Path,
    built: &mut HashMap<String, Hash, S>,
) -> Result<[u8; 32]> {
    let cargo_path = input_dir.join(rel);
    let crate_dir = cargo_path
        .parent()
        .context("Cargo.toml has no parent dir")?;
    let hash = build_wasm_for_crate(crate_dir, out_dir, built)?;
    Ok(*hash.as_bytes())
}

fn compile_image(img: &HsdxImage, input_dir: &Path, out_dir: &Path) -> Result<ImageAttr> {
    let abs = resolve(input_dir, &img.data)?;
    let bytes = std::fs::read(&abs).with_context(|| format!("reading image {}", abs.display()))?;
    let hash = write_blob(out_dir, &bytes)?;
    Ok(ImageAttr {
        data:           ByteArray::new(*hash.as_bytes()),
        address_mode_u: (img.address_mode_u),
        address_mode_v: (img.address_mode_v),
        address_mode_w: (img.address_mode_w),
        mag_filter:     (img.mag_filter),
        min_filter:     (img.min_filter),
        mipmap_filter:  (img.mipmap_filter),
        srgb:           (img.srgb),
    })
}

fn compile_collider(c: &HsdxCollider) -> Result<ColliderAttr> {
    Ok(match c {
        HsdxCollider::Capsule { height, radius } => ColliderAttr::Capsule {
            height: *height,
            radius: *radius,
        },
        HsdxCollider::ConvexHull(_) => {
            bail!("convex hull colliders are not supported in .hsdx source files");
        }
        HsdxCollider::Trimesh { .. } => {
            bail!("trimesh colliders are not supported in .hsdx source files");
        }
        HsdxCollider::Cuboid { x, y, z } => ColliderAttr::Cuboid {
            x: *x,
            y: *y,
            z: *z,
        },
        HsdxCollider::Cylinder { height, radius } => ColliderAttr::Cylinder {
            height: *height,
            radius: *radius,
        },
        HsdxCollider::Sphere(r) => ColliderAttr::Sphere(*r),
    })
}

fn compile_material(m: &HsdxMaterial) -> MaterialAttr {
    MaterialAttr {
        alpha_cutoff:               (m.alpha_cutoff),
        alpha_mode:                 (m.alpha_mode.clone()),
        base_color:                 (m.base_color.clone().map(ColorVec)),
        base_color_texture:         (m.base_color_texture.clone()),
        double_sided:               (m.double_sided),
        emissive:                   (m.emissive.clone().map(ColorVec)),
        emissive_texture:           (m.emissive_texture.clone()),
        metallic:                   (m.metallic),
        metallic_roughness_texture: (m.metallic_roughness_texture.clone()),
        normal_texture:             (m.normal_texture.clone()),
        occlusion_texture:          (m.occlusion_texture.clone()),
        roughness:                  (m.roughness),
    }
}

fn compile_rigid_body(rb: &HsdxRigidBody) -> Result<RigidBodyAttr> {
    let kind = match rb.kind.as_str() {
        "Static" => RigidBodyKind::Static,
        "Kinematic" => RigidBodyKind::Kinematic,
        "Dynamic" => RigidBodyKind::Dynamic,
        other => bail!("unknown rigid body kind {other:?}; expected Static, Kinematic, or Dynamic"),
    };
    Ok(RigidBodyAttr {
        kind:            Some(kind),
        angular_damping: (rb.angular_damping),
        friction:        (rb.friction),
        linear_damping:  (rb.linear_damping),
        mass:            (rb.mass),
        restitution:     (rb.restitution),
    })
}

fn compile_xform(x: &HsdxXform) -> XformAttr {
    let mut out = XformAttr::default();
    if let Some(t) = x.translation.clone() {
        out.translation.copy_from_slice(&t);
    }
    if let Some(r) = x.rotation.clone() {
        out.rotation.copy_from_slice(&r);
    }
    if let Some(s) = x.scale.clone() {
        out.scale.copy_from_slice(&s);
    }
    out
}

fn resolve(base: &Path, rel: &str) -> Result<PathBuf> {
    std::fs::canonicalize(base.join(rel)).with_context(|| format!("resolving path {rel}"))
}
