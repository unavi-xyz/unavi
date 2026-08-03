use std::{
    collections::BTreeMap,
    sync::Arc,
};

use anyhow::bail;
use bevy::{
    math::{
        Affine3A,
        Quat,
        Vec3,
    },
    transform::components::GlobalTransform,
};
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{
        Attribute,
        asset::AssetAttr,
        attributes_map,
        collider::ColliderAttr,
        gravity_scale::GravityScaleAttr,
        image::ImageAttr,
        material::{
            ColorVec,
            MaterialAttr,
        },
        mesh::{
            MeshAttr,
            Topology,
        },
        name::NameAttr,
        portal::{
            PortalAttr,
            PortalDestination,
            PortalReceptor,
        },
        relationships_map,
        rigid_body::{
            RigidBodyAttr,
            RigidBodyKind,
        },
        spawn::SpawnAttr,
        xform::XformAttr,
    },
};
use iroh_docs::NamespaceId;
use loro::{
    LoroDoc,
    LoroMap,
    TreeID,
    TreeParentId,
};
use loro_surgeon::bytes::ByteArray;
use unavi_quota::{
    Flow,
    limits::{
        MAX_MESH_ELEMENTS,
        MAX_NAME_BYTES,
    },
};

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api,
        registry::{
            firewall::validate_firewall,
            transform::{
                AbsoluteNodeId,
                DOC_ROOT_TRANSFORM_REGISTRY,
                NODE_TRANSFORM_REGISTRY,
            },
        },
        wired::scene::util::{
            f32s_to_bytes,
            u32s_to_bytes,
        },
    },
};

#[derive(Clone)]
pub struct PrimRes {
    pub doc:      Arc<LoroDoc>,
    pub doc_id:   NamespaceId,
    pub id:       TreeID,
    /// Proxy prims (e.g. agent bone nodes) are read-only from scripts.
    pub is_proxy: bool,
}

#[derive(Clone, Copy, Default)]
pub struct PrimColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Clone, Copy)]
pub enum PrimAlphaMode {
    Add,
    Blend,
    Mask,
    Multiply,
    Opaque,
    PreMultiplied,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum PrimTopology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

pub struct PrimMesh {
    pub topology:   PrimTopology,
    pub attributes: Vec<(String, [u8; 32])>,
    pub indices:    Option<[u8; 32]>,
}

#[derive(Default)]
pub struct PrimMaterial {
    pub alpha_cutoff:               Option<f32>,
    pub alpha_mode:                 Option<PrimAlphaMode>,
    pub base_color:                 Option<PrimColor>,
    pub base_color_texture:         Option<String>,
    pub double_sided:               Option<bool>,
    pub emissive:                   Option<PrimColor>,
    pub emissive_texture:           Option<String>,
    pub metallic:                   Option<f32>,
    pub metallic_roughness_texture: Option<String>,
    pub normal_texture:             Option<String>,
    pub occlusion_texture:          Option<String>,
    pub roughness:                  Option<f32>,
}

pub struct PrimImage {
    pub data:           [u8; 32],
    pub address_mode_u: Option<i32>,
    pub address_mode_v: Option<i32>,
    pub address_mode_w: Option<i32>,
    pub mag_filter:     Option<i32>,
    pub min_filter:     Option<i32>,
    pub mipmap_filter:  Option<i32>,
    pub srgb:           Option<bool>,
}

pub enum PrimCollider {
    Capsule {
        height: f32,
        radius: f32,
    },
    ConvexHull([u8; 32]),
    Cuboid([f32; 3]),
    Cylinder {
        height: f32,
        radius: f32,
    },
    Sphere(f32),
    Trimesh {
        indices:  [u8; 32],
        vertices: [u8; 32],
    },
}

#[derive(Default)]
pub struct PrimRigidBody {
    pub kind:            PrimRigidBodyKind,
    pub angular_damping: Option<f32>,
    pub friction:        Option<f32>,
    pub linear_damping:  Option<f32>,
    pub mass:            Option<f32>,
    pub restitution:     Option<f32>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum PrimRigidBodyKind {
    #[default]
    Dynamic,
    Kinematic,
    Static,
}

pub struct PrimPortalReceptor {
    pub document: [u8; 32],
    pub prim:     String,
}

pub struct PrimPortalDestination {
    pub receptor: Option<PrimPortalReceptor>,
    pub space:    [u8; 32],
}

pub struct PrimPortal {
    pub destination: Option<PrimPortalDestination>,
    pub size_x:      f32,
    pub size_y:      f32,
}

pub struct PrimSpawn {
    pub radius: f32,
}

async fn get_prim(api: &Api, rep: u32) -> anyhow::Result<PrimRes> {
    api.wired_scene
        .lock()
        .await
        .prims
        .get(rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid prim rep: {rep}"))
}

fn prim_meta(doc: &LoroDoc, id: TreeID) -> anyhow::Result<LoroMap> {
    doc.get_tree(&*HSD_CONTAINER_ID)
        .get_meta(id)
        .map_err(Into::into)
}

fn attrs_or_create(meta: &LoroMap) -> anyhow::Result<LoroMap> {
    meta.ensure_mergeable_map("attributes").map_err(Into::into)
}

fn rels_or_create(meta: &LoroMap) -> anyhow::Result<LoroMap> {
    meta.ensure_mergeable_map("relationships")
        .map_err(Into::into)
}

fn write_attr<A: Attribute>(meta: &LoroMap, attr: &A) -> anyhow::Result<()> {
    let attrs = attrs_or_create(meta)?;
    attr.attr_reconcile(attrs).map_err(Into::into)
}

fn clear_attr(meta: &LoroMap, key: &str) -> anyhow::Result<()> {
    if let Some(attrs) = attributes_map(meta) {
        attrs.delete(key)?;
    }
    Ok(())
}

fn read_attr<A: Attribute>(meta: &LoroMap) -> Option<A> {
    A::attr_hydrate(&attributes_map(meta)?).ok()
}

fn ensure_writable(api: &Api, prim: &PrimRes) -> anyhow::Result<()> {
    if prim.is_proxy {
        bail!("cannot write proxy prim")
    }
    let caller_is_system = api
        .permissions
        .contains(&crate::permissions::ApiName::System);
    if !caller_is_system && unavi_space::membership::doc_space(api.doc_id).is_none() {
        bail!("caller document is not placed in a space")
    }
    if api.doc_id != prim.doc_id
        && !caller_is_system
        && !unavi_space::membership::same_space(api.doc_id, prim.doc_id)
    {
        bail!("cross-document write requires both documents in the same space")
    }
    validate_firewall(&api.doc_id, &prim.doc_id, Channel::SceneWrite)
}

const fn maybe<T>(value: Option<T>) -> Option<T> {
    value
}

const fn from_maybe<T>(value: Option<T>) -> Option<T> {
    value
}

pub async fn clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .lock()
        .await
        .prims
        .insert_clone(rep, &api.quota)
        .ok_or_else(|| anyhow::anyhow!("invalid prim"))?
        .map_err(Into::into)
}

pub async fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.lock().await.prims.remove(rep);
    Ok(())
}

pub async fn id(api: &Api, rep: u32) -> anyhow::Result<String> {
    Ok(get_prim(api, rep).await?.id.to_string())
}

pub async fn parent(api: &Api, rep: u32) -> anyhow::Result<Option<u32>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let tree = prim.doc.get_tree(&*HSD_CONTAINER_ID);
    let Some(TreeParentId::Node(parent_id)) = tree.parent(prim.id) else {
        return Ok(None);
    };
    let mut scene = api.wired_scene.lock().await;
    Ok(Some(scene.prims.insert(
        PrimRes {
            doc:      prim.doc,
            doc_id:   prim.doc_id,
            id:       parent_id,
            is_proxy: prim.is_proxy,
        },
        &api.quota,
    )?))
}

pub async fn children(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(Vec::new());
    }
    let tree = prim.doc.get_tree(&*HSD_CONTAINER_ID);
    let child_ids = tree.children(prim.id).unwrap_or_default();
    let mut scene = api.wired_scene.lock().await;
    Ok(child_ids
        .into_iter()
        .map(|id| {
            scene.prims.insert(
                PrimRes {
                    doc: Arc::clone(&prim.doc),
                    doc_id: prim.doc_id,
                    id,
                    is_proxy: prim.is_proxy,
                },
                &api.quota,
            )
        })
        .collect::<Result<Vec<_>, _>>()?)
}

pub async fn add_child(api: &Api, self_rep: u32, child_rep: u32) -> anyhow::Result<()> {
    let (parent, child) = {
        let scene = api.wired_scene.lock().await;
        let parent = scene
            .prims
            .get(self_rep)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("invalid parent rep: {self_rep}"))?;
        let child = scene
            .prims
            .get(child_rep)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("invalid child rep: {child_rep}"))?;
        drop(scene);
        (parent, child)
    };
    ensure_writable(api, &parent)?;
    if child.is_proxy {
        bail!("cannot add proxy prim as child")
    }
    anyhow::ensure!(
        Arc::ptr_eq(&parent.doc, &child.doc),
        "prims must belong to the same document"
    );
    let tree = parent.doc.get_tree(&*HSD_CONTAINER_ID);
    tree.mov(child.id, TreeParentId::Node(parent.id))?;
    Ok(())
}

pub async fn remove_child(api: &Api, self_rep: u32, child_rep: u32) -> anyhow::Result<()> {
    let (parent, child) = {
        let scene = api.wired_scene.lock().await;
        let parent = scene
            .prims
            .get(self_rep)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("invalid parent rep: {self_rep}"))?;
        let child = scene
            .prims
            .get(child_rep)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("invalid child rep: {child_rep}"))?;
        drop(scene);
        (parent, child)
    };
    ensure_writable(api, &parent)?;
    if child.is_proxy {
        bail!("cannot remove proxy prim as child")
    }
    let tree = child.doc.get_tree(&*HSD_CONTAINER_ID);
    tree.mov(child.id, TreeParentId::Root)?;
    Ok(())
}

pub async fn name(api: &Api, rep: u32) -> anyhow::Result<Option<String>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<NameAttr>(&meta).map(|n| n.0))
}

pub async fn set_name(api: &Api, rep: u32, value: Option<String>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    if let Some(s) = &value {
        anyhow::ensure!(s.len() <= MAX_NAME_BYTES, "name too long");
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(s) => write_attr(&meta, &NameAttr(s))?,
        None => clear_attr(&meta, NameAttr::KEY)?,
    }
    Ok(())
}

pub async fn asset(api: &Api, rep: u32) -> anyhow::Result<Option<Vec<u8>>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<AssetAttr>(&meta).map(|a| a.0.0.to_vec()))
}

pub async fn set_asset(api: &Api, rep: u32, value: Option<Vec<u8>>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(bytes) => {
            let arr: [u8; 32] = bytes
                .as_slice()
                .try_into()
                .map_err(|_| anyhow::anyhow!("asset blob id must be 32 bytes"))?;
            write_attr(&meta, &AssetAttr(ByteArray::new(arr)))?;
        }
        None => clear_attr(&meta, AssetAttr::KEY)?,
    }
    Ok(())
}

pub async fn xform(api: &Api, rep: u32) -> anyhow::Result<Option<XformAttr>> {
    let prim = get_prim(api, rep).await?;
    let local = NODE_TRANSFORM_REGISTRY
        .read()
        .get(&AbsoluteNodeId {
            doc:  prim.doc_id,
            node: prim.id,
        })
        .map(|v| v.local);
    if let Some(t) = local {
        return Ok(Some(XformAttr {
            translation: t.translation.to_array(),
            rotation:    [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
            scale:       t.scale.to_array(),
        }));
    }
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<XformAttr>(&meta))
}

pub async fn set_xform(api: &Api, rep: u32, value: Option<XformAttr>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(x) => {
            write_attr(&meta, &x)?;
            if let Some(v) = NODE_TRANSFORM_REGISTRY.write().get_mut(&AbsoluteNodeId {
                doc:  prim.doc_id,
                node: prim.id,
            }) {
                v.local.translation = Vec3::from_array(x.translation);
                v.local.rotation = Quat::from_array(x.rotation);
                v.local.scale = Vec3::from_array(x.scale);
            }
        }
        None => clear_attr(&meta, XformAttr::KEY)?,
    }
    Ok(())
}

pub async fn global_xform(api: &Api, rep: u32) -> anyhow::Result<XformAttr> {
    let prim = get_prim(api, rep).await?;
    let node = |node| AbsoluteNodeId {
        doc: prim.doc_id,
        node,
    };
    let world = if prim.is_proxy {
        NODE_TRANSFORM_REGISTRY
            .read()
            .get(&node(prim.id))
            .map_or(Affine3A::IDENTITY, |s| s.world.affine())
    } else {
        let tree = prim.doc.get_tree(&*HSD_CONTAINER_ID);
        let mut local = Affine3A::IDENTITY;
        let mut cur = Some(prim.id);
        while let Some(id) = cur {
            let t = NODE_TRANSFORM_REGISTRY
                .read()
                .get(&node(id))
                .map(|s| s.local)
                .unwrap_or_default();
            local = t.compute_affine() * local;
            cur = match tree.parent(id) {
                Some(TreeParentId::Node(p)) => Some(p),
                _ => None,
            };
        }
        let root = DOC_ROOT_TRANSFORM_REGISTRY
            .read()
            .get(&prim.doc_id)
            .map_or(Affine3A::IDENTITY, GlobalTransform::affine);
        root * local
    };
    let (sc, ro, tr) = world.to_scale_rotation_translation();
    Ok(XformAttr {
        translation: [tr.x, tr.y, tr.z],
        rotation:    [ro.x, ro.y, ro.z, ro.w],
        scale:       [sc.x, sc.y, sc.z],
    })
}

pub async fn gravity_scale(api: &Api, rep: u32) -> anyhow::Result<f32> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(1.0);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<GravityScaleAttr>(&meta).map_or(1.0, |g| g.scale as f32))
}

pub async fn set_gravity_scale(api: &Api, rep: u32, value: f32) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    write_attr(
        &meta,
        &GravityScaleAttr {
            scale: f64::from(value),
        },
    )?;
    Ok(())
}

pub async fn mesh(api: &Api, rep: u32) -> anyhow::Result<Option<PrimMesh>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<MeshAttr>(&meta).map(mesh_attr_to_prim))
}

pub async fn set_mesh(api: &Api, rep: u32, value: Option<PrimMesh>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(m) => write_attr(&meta, &prim_to_mesh_attr(m))?,
        None => clear_attr(&meta, MeshAttr::KEY)?,
    }
    Ok(())
}

pub async fn set_mesh_stream(
    api: &Api,
    rep: u32,
    key: String,
    values: Option<Vec<f32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    let mut attr = read_attr::<MeshAttr>(&meta).unwrap_or_else(|| MeshAttr {
        topology:   Topology::TriangleList,
        attributes: BTreeMap::new(),
        indices:    None,
    });
    match values {
        Some(v) => {
            anyhow::ensure!(v.len() <= MAX_MESH_ELEMENTS, "mesh stream too large");
            anyhow::ensure!(key.len() <= MAX_NAME_BYTES, "mesh attribute key too long");
            api.quota.spend(Flow::BlobUpload, 1.0)?;
            let hash = super::upload_blob(f32s_to_bytes(&v)).await?;
            attr.attributes
                .insert(key, ByteArray::new(*hash.as_bytes()));
        }
        None => {
            attr.attributes.remove(&key);
        }
    }
    write_attr(&meta, &attr)?;
    Ok(())
}

pub async fn set_mesh_indices_u32(
    api: &Api,
    rep: u32,
    values: Option<Vec<u32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    let mut attr = read_attr::<MeshAttr>(&meta).unwrap_or_else(|| MeshAttr {
        topology:   Topology::TriangleList,
        attributes: BTreeMap::new(),
        indices:    None,
    });
    attr.indices = match values {
        Some(v) => {
            anyhow::ensure!(v.len() <= MAX_MESH_ELEMENTS, "mesh indices too large");
            api.quota.spend(Flow::BlobUpload, 1.0)?;
            let hash = super::upload_blob(u32s_to_bytes(&v)).await?;
            Some(ByteArray::new(*hash.as_bytes()))
        }
        None => None,
    };
    write_attr(&meta, &attr)?;
    Ok(())
}

fn mesh_attr_to_prim(attr: MeshAttr) -> PrimMesh {
    PrimMesh {
        topology:   topology_to_prim(&attr.topology),
        attributes: attr.attributes.into_iter().map(|(k, v)| (k, v.0)).collect(),
        indices:    from_maybe(attr.indices).map(|b| b.0),
    }
}

fn prim_to_mesh_attr(m: PrimMesh) -> MeshAttr {
    MeshAttr {
        topology:   topology_from_prim(m.topology),
        attributes: m
            .attributes
            .into_iter()
            .map(|(k, v)| (k, ByteArray::new(v)))
            .collect(),
        indices:    maybe(m.indices.map(ByteArray::new)),
    }
}

const fn topology_to_prim(t: &Topology) -> PrimTopology {
    match t {
        Topology::PointList => PrimTopology::PointList,
        Topology::LineList => PrimTopology::LineList,
        Topology::LineStrip => PrimTopology::LineStrip,
        Topology::TriangleList => PrimTopology::TriangleList,
        Topology::TriangleStrip => PrimTopology::TriangleStrip,
    }
}

const fn topology_from_prim(t: PrimTopology) -> Topology {
    match t {
        PrimTopology::PointList => Topology::PointList,
        PrimTopology::LineList => Topology::LineList,
        PrimTopology::LineStrip => Topology::LineStrip,
        PrimTopology::TriangleList => Topology::TriangleList,
        PrimTopology::TriangleStrip => Topology::TriangleStrip,
    }
}

pub async fn material(api: &Api, rep: u32) -> anyhow::Result<Option<PrimMaterial>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<MaterialAttr>(&meta).map(material_attr_to_prim))
}

pub async fn set_material(api: &Api, rep: u32, value: Option<PrimMaterial>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(m) => write_attr(&meta, &prim_to_material_attr(m))?,
        None => clear_attr(&meta, MaterialAttr::KEY)?,
    }
    Ok(())
}

fn material_attr_to_prim(attr: MaterialAttr) -> PrimMaterial {
    PrimMaterial {
        alpha_cutoff:               from_maybe(attr.alpha_cutoff).map(|v| v as f32),
        alpha_mode:                 from_maybe(attr.alpha_mode).and_then(|s| match s.as_str() {
            "add" => Some(PrimAlphaMode::Add),
            "blend" => Some(PrimAlphaMode::Blend),
            "mask" => Some(PrimAlphaMode::Mask),
            "multiply" => Some(PrimAlphaMode::Multiply),
            "opaque" => Some(PrimAlphaMode::Opaque),
            "premultiplied" => Some(PrimAlphaMode::PreMultiplied),
            _ => None,
        }),
        base_color:                 from_maybe(attr.base_color).map(color_vec_to_prim),
        base_color_texture:         from_maybe(attr.base_color_texture),
        double_sided:               from_maybe(attr.double_sided),
        emissive:                   from_maybe(attr.emissive).map(color_vec_to_prim),
        emissive_texture:           from_maybe(attr.emissive_texture),
        metallic:                   from_maybe(attr.metallic).map(|v| v as f32),
        metallic_roughness_texture: from_maybe(attr.metallic_roughness_texture),
        normal_texture:             from_maybe(attr.normal_texture),
        occlusion_texture:          from_maybe(attr.occlusion_texture),
        roughness:                  from_maybe(attr.roughness).map(|v| v as f32),
    }
}

fn prim_to_material_attr(m: PrimMaterial) -> MaterialAttr {
    MaterialAttr {
        alpha_cutoff:               maybe(m.alpha_cutoff.map(f64::from)),
        alpha_mode:                 maybe(m.alpha_mode.map(|mode| {
            match mode {
                PrimAlphaMode::Add => "add",
                PrimAlphaMode::Blend => "blend",
                PrimAlphaMode::Mask => "mask",
                PrimAlphaMode::Multiply => "multiply",
                PrimAlphaMode::Opaque => "opaque",
                PrimAlphaMode::PreMultiplied => "premultiplied",
            }
            .to_string()
        })),
        base_color:                 maybe(m.base_color.map(prim_color_to_vec)),
        base_color_texture:         maybe(m.base_color_texture),
        double_sided:               maybe(m.double_sided),
        emissive:                   maybe(m.emissive.map(prim_color_to_vec)),
        emissive_texture:           maybe(m.emissive_texture),
        metallic:                   maybe(m.metallic.map(f64::from)),
        metallic_roughness_texture: maybe(m.metallic_roughness_texture),
        normal_texture:             maybe(m.normal_texture),
        occlusion_texture:          maybe(m.occlusion_texture),
        roughness:                  maybe(m.roughness.map(f64::from)),
    }
}

fn color_vec_to_prim(c: ColorVec) -> PrimColor {
    let v = c.0;
    PrimColor {
        r: v.first().copied().unwrap_or(1.0) as f32,
        g: v.get(1).copied().unwrap_or(1.0) as f32,
        b: v.get(2).copied().unwrap_or(1.0) as f32,
        a: v.get(3).copied().unwrap_or(1.0) as f32,
    }
}

fn prim_color_to_vec(c: PrimColor) -> ColorVec {
    ColorVec(vec![
        f64::from(c.r),
        f64::from(c.g),
        f64::from(c.b),
        f64::from(c.a),
    ])
}

pub async fn image(api: &Api, rep: u32) -> anyhow::Result<Option<PrimImage>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<ImageAttr>(&meta).map(image_attr_to_prim))
}

pub async fn set_image(api: &Api, rep: u32, value: Option<PrimImage>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(img) => write_attr(&meta, &prim_to_image_attr(img))?,
        None => clear_attr(&meta, ImageAttr::KEY)?,
    }
    Ok(())
}

fn image_attr_to_prim(attr: ImageAttr) -> PrimImage {
    PrimImage {
        data:           attr.data.0,
        address_mode_u: from_maybe(attr.address_mode_u).map(|v| v as i32),
        address_mode_v: from_maybe(attr.address_mode_v).map(|v| v as i32),
        address_mode_w: from_maybe(attr.address_mode_w).map(|v| v as i32),
        mag_filter:     from_maybe(attr.mag_filter).map(|v| v as i32),
        min_filter:     from_maybe(attr.min_filter).map(|v| v as i32),
        mipmap_filter:  from_maybe(attr.mipmap_filter).map(|v| v as i32),
        srgb:           from_maybe(attr.srgb),
    }
}

fn prim_to_image_attr(img: PrimImage) -> ImageAttr {
    ImageAttr {
        address_mode_u: maybe(img.address_mode_u.map(i64::from)),
        address_mode_v: maybe(img.address_mode_v.map(i64::from)),
        address_mode_w: maybe(img.address_mode_w.map(i64::from)),
        data:           ByteArray::new(img.data),
        mag_filter:     maybe(img.mag_filter.map(i64::from)),
        min_filter:     maybe(img.min_filter.map(i64::from)),
        mipmap_filter:  maybe(img.mipmap_filter.map(i64::from)),
        srgb:           maybe(img.srgb),
    }
}

pub async fn collider(api: &Api, rep: u32) -> anyhow::Result<Option<PrimCollider>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<ColliderAttr>(&meta).map(|c| match c {
        ColliderAttr::Capsule { height, radius } => PrimCollider::Capsule {
            height: height as f32,
            radius: radius as f32,
        },
        ColliderAttr::ConvexHull(hash) => PrimCollider::ConvexHull(hash.0),
        ColliderAttr::Cuboid { x, y, z } => PrimCollider::Cuboid([x as f32, y as f32, z as f32]),
        ColliderAttr::Cylinder { height, radius } => PrimCollider::Cylinder {
            height: height as f32,
            radius: radius as f32,
        },
        ColliderAttr::Sphere(r) => PrimCollider::Sphere(r as f32),
        ColliderAttr::Trimesh { indices, vertices } => PrimCollider::Trimesh {
            indices:  indices.0,
            vertices: vertices.0,
        },
    }))
}

pub async fn set_collider(api: &Api, rep: u32, value: Option<PrimCollider>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(c) => {
            let attr = match c {
                PrimCollider::Capsule { height, radius } => ColliderAttr::Capsule {
                    height: f64::from(height),
                    radius: f64::from(radius),
                },
                PrimCollider::ConvexHull(hash) => ColliderAttr::ConvexHull(ByteArray::new(hash)),
                PrimCollider::Cuboid([x, y, z]) => ColliderAttr::Cuboid {
                    x: f64::from(x),
                    y: f64::from(y),
                    z: f64::from(z),
                },
                PrimCollider::Cylinder { height, radius } => ColliderAttr::Cylinder {
                    height: f64::from(height),
                    radius: f64::from(radius),
                },
                PrimCollider::Sphere(r) => ColliderAttr::Sphere(f64::from(r)),
                PrimCollider::Trimesh { indices, vertices } => ColliderAttr::Trimesh {
                    indices:  ByteArray::new(indices),
                    vertices: ByteArray::new(vertices),
                },
            };
            write_attr(&meta, &attr)?;
        }
        None => clear_attr(&meta, ColliderAttr::KEY)?,
    }
    Ok(())
}

pub async fn rigid_body(api: &Api, rep: u32) -> anyhow::Result<Option<PrimRigidBody>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<RigidBodyAttr>(&meta).map(rigid_body_attr_to_prim))
}

pub async fn set_rigid_body(
    api: &Api,
    rep: u32,
    value: Option<PrimRigidBody>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(rb) => write_attr(&meta, &prim_to_rigid_body_attr(rb))?,
        None => clear_attr(&meta, RigidBodyAttr::KEY)?,
    }
    Ok(())
}

fn rigid_body_attr_to_prim(attr: RigidBodyAttr) -> PrimRigidBody {
    PrimRigidBody {
        kind:            match attr.kind.unwrap_or(RigidBodyKind::Dynamic) {
            RigidBodyKind::Dynamic => PrimRigidBodyKind::Dynamic,
            RigidBodyKind::Kinematic => PrimRigidBodyKind::Kinematic,
            RigidBodyKind::Static => PrimRigidBodyKind::Static,
        },
        angular_damping: from_maybe(attr.angular_damping).map(|v| v as f32),
        friction:        from_maybe(attr.friction).map(|v| v as f32),
        linear_damping:  from_maybe(attr.linear_damping).map(|v| v as f32),
        mass:            from_maybe(attr.mass).map(|v| v as f32),
        restitution:     from_maybe(attr.restitution).map(|v| v as f32),
    }
}

fn prim_to_rigid_body_attr(rb: PrimRigidBody) -> RigidBodyAttr {
    RigidBodyAttr {
        kind:            Some(match rb.kind {
            PrimRigidBodyKind::Dynamic => RigidBodyKind::Dynamic,
            PrimRigidBodyKind::Kinematic => RigidBodyKind::Kinematic,
            PrimRigidBodyKind::Static => RigidBodyKind::Static,
        }),
        angular_damping: maybe(rb.angular_damping.map(f64::from)),
        friction:        maybe(rb.friction.map(f64::from)),
        linear_damping:  maybe(rb.linear_damping.map(f64::from)),
        mass:            maybe(rb.mass.map(f64::from)),
        restitution:     maybe(rb.restitution.map(f64::from)),
    }
}

pub async fn portal(api: &Api, rep: u32) -> anyhow::Result<Option<PrimPortal>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<PortalAttr>(&meta).map(portal_attr_to_prim))
}

pub async fn set_portal(api: &Api, rep: u32, value: Option<PrimPortal>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(p) => write_attr(&meta, &prim_portal_to_attr(p))?,
        None => clear_attr(&meta, PortalAttr::KEY)?,
    }
    Ok(())
}

fn prim_portal_to_attr(p: PrimPortal) -> PortalAttr {
    PortalAttr {
        destination: p.destination.map(|d| PortalDestination {
            receptor: d.receptor.map(|r| PortalReceptor {
                document: ByteArray(r.document),
                prim:     r.prim,
            }),
            space:    ByteArray(d.space),
        }),
        size_x:      f64::from(p.size_x),
        size_y:      f64::from(p.size_y),
    }
}

fn portal_attr_to_prim(attr: PortalAttr) -> PrimPortal {
    PrimPortal {
        destination: attr.destination.map(|d| PrimPortalDestination {
            receptor: d.receptor.map(|r| PrimPortalReceptor {
                document: r.document.0,
                prim:     r.prim,
            }),
            space:    d.space.0,
        }),
        size_x:      attr.size_x as f32,
        size_y:      attr.size_y as f32,
    }
}

pub async fn spawn(api: &Api, rep: u32) -> anyhow::Result<Option<PrimSpawn>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    Ok(read_attr::<SpawnAttr>(&meta).map(|a| PrimSpawn {
        radius: a.radius as f32,
    }))
}

pub async fn set_spawn(api: &Api, rep: u32, value: Option<PrimSpawn>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let meta = prim_meta(&prim.doc, prim.id)?;
    match value {
        Some(s) => write_attr(
            &meta,
            &SpawnAttr {
                radius: f64::from(s.radius),
            },
        )?,
        None => clear_attr(&meta, SpawnAttr::KEY)?,
    }
    Ok(())
}

pub async fn relationships(api: &Api, rep: u32) -> anyhow::Result<Vec<(String, String)>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(Vec::new());
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    let Some(rels) = relationships_map(&meta) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    rels.for_each(|k, v| {
        if let loro::ValueOrContainer::Value(loro::LoroValue::String(s)) = v {
            out.push((k.to_string(), s.to_string()));
        }
    });
    Ok(out)
}

pub async fn get_relationship(api: &Api, rep: u32, key: String) -> anyhow::Result<Option<String>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let meta = prim_meta(&prim.doc, prim.id)?;
    let Some(rels) = relationships_map(&meta) else {
        return Ok(None);
    };
    match rels.get(&key) {
        Some(loro::ValueOrContainer::Value(loro::LoroValue::String(s))) => Ok(Some(s.to_string())),
        _ => Ok(None),
    }
}

pub async fn set_relationship(
    api: &Api,
    rep: u32,
    key: String,
    target: Option<String>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    anyhow::ensure!(key.len() <= MAX_NAME_BYTES, "relationship key too long");
    let meta = prim_meta(&prim.doc, prim.id)?;
    match target {
        Some(target_id) => {
            let tree_id = TreeID::try_from(target_id.as_str())
                .map_err(|_| anyhow::anyhow!("invalid prim id: {target_id}"))?;
            let tree = prim.doc.get_tree(&*HSD_CONTAINER_ID);
            anyhow::ensure!(
                tree.contains(tree_id),
                "relationship target does not exist in this document"
            );
            let rels = rels_or_create(&meta)?;
            rels.insert(&key, target_id)?;
        }
        None => {
            if let Some(rels) = relationships_map(&meta) {
                rels.delete(&key)?;
            }
        }
    }
    Ok(())
}
