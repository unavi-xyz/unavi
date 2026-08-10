use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        Mutex,
    },
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
    attributes::{
        Attribute,
        collider::ColliderAttr,
        gravity_scale::GravityScaleAttr,
        image::ImageAttr,
        material::{
            ColorVec,
            MaterialAttr,
        },
        material_graph::{
            MAX_PUBLIC_INPUTS,
            overrides::GraphOverridesAttr,
            value::{
                GraphValue,
                is_finite,
            },
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
        rigid_body::{
            RigidBodyAttr,
            RigidBodyKind,
        },
        slots,
        spawn::SpawnAttr,
        xform::XformAttr,
    },
    id::{
        DocId,
        PrimId,
    },
    property::{
        Parent,
        Property,
    },
    state::SceneState,
};
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
    pub state:    Arc<Mutex<SceneState>>,
    pub doc_id:   DocId,
    pub id:       PrimId,
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

/// Runtime mirror of the graph format's `GraphValue`, kept separate so the
/// two WIT backends share one conversion rather than each writing their own.
#[derive(Clone, Copy)]
pub enum PrimGraphValue {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Color(PrimColor),
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
    pub topology: PrimTopology,
}

#[derive(Default)]
pub struct PrimMaterial {
    pub alpha_cutoff: Option<f32>,
    pub alpha_mode:   Option<PrimAlphaMode>,
    pub base_color:   Option<PrimColor>,
    pub double_sided: Option<bool>,
    pub emissive:     Option<PrimColor>,
    pub metallic:     Option<f32>,
    pub roughness:    Option<f32>,
}

pub struct PrimImage {
    pub address_mode_u: Option<i32>,
    pub address_mode_v: Option<i32>,
    pub address_mode_w: Option<i32>,
    pub mag_filter:     Option<i32>,
    pub min_filter:     Option<i32>,
    pub mipmap_filter:  Option<i32>,
    pub srgb:           Option<bool>,
}

pub enum PrimCollider {
    Capsule { height: f32, radius: f32 },
    ConvexHull,
    Cuboid([f32; 3]),
    Cylinder { height: f32, radius: f32 },
    Sphere(f32),
    Trimesh,
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

impl PrimRes {
    /// Writes are synchronous and land in state only. Nothing here touches
    /// storage, which is what makes a spawn/despawn loop free and keeps the
    /// host off `AsyncCommands` on the hot path.
    fn with<T>(&self, f: impl FnOnce(&mut SceneState) -> T) -> anyhow::Result<T> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("scene state poisoned"))?;
        Ok(f(&mut state))
    }

    fn read_attr<A: Attribute>(&self) -> anyhow::Result<Option<A>> {
        self.with(|state| state.attribute::<A>(self.id).and_then(Result::ok))
    }

    fn write_attr<A: Attribute>(&self, value: &A) -> anyhow::Result<()> {
        self.with(|state| state.set_attribute(self.id, value))??;
        Ok(())
    }

    fn clear(&self, name: &str) -> anyhow::Result<()> {
        self.with(|state| state.remove_property(self.id, name))
    }

    fn write_or_clear<A: Attribute>(&self, value: Option<A>) -> anyhow::Result<()> {
        value.map_or_else(|| self.clear(A::KEY), |attr| self.write_attr(&attr))
    }

    fn slot(&self, name: &str) -> anyhow::Result<Option<Vec<u8>>> {
        self.with(|state| {
            state
                .get(self.id)
                .and_then(|p| p.slot(name))
                .map(ToOwned::to_owned)
        })
    }

    fn set_slot(&self, name: &str, value: Option<Vec<u8>>) -> anyhow::Result<()> {
        self.with(|state| {
            let Some(value) = value else {
                state.remove_slot(self.id, name);
                return Ok(());
            };
            state.set_slot(self.id, name, value)
        })??;
        Ok(())
    }
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
    let Some(parent_id) = prim.with(|state| state.parent(prim.id))? else {
        return Ok(None);
    };
    let mut scene = api.wired_scene.lock().await;
    Ok(Some(scene.prims.insert(
        PrimRes {
            id: parent_id,
            ..prim
        },
        &api.quota,
    )?))
}

pub async fn children(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(Vec::new());
    }
    let child_ids = prim.with(|state| state.children(prim.id))?;
    let mut scene = api.wired_scene.lock().await;
    Ok(child_ids
        .into_iter()
        .map(|id| {
            scene.prims.insert(
                PrimRes {
                    state: Arc::clone(&prim.state),
                    doc_id: prim.doc_id,
                    id,
                    is_proxy: prim.is_proxy,
                },
                &api.quota,
            )
        })
        .collect::<Result<Vec<_>, _>>()?)
}

async fn pair(api: &Api, self_rep: u32, child_rep: u32) -> anyhow::Result<(PrimRes, PrimRes)> {
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
    Ok((parent, child))
}

pub async fn add_child(api: &Api, self_rep: u32, child_rep: u32) -> anyhow::Result<()> {
    let (parent, child) = pair(api, self_rep, child_rep).await?;
    ensure_writable(api, &parent)?;
    if child.is_proxy {
        bail!("cannot add proxy prim as child")
    }
    anyhow::ensure!(
        Arc::ptr_eq(&parent.state, &child.state),
        "prims must belong to the same document"
    );
    child.with(|state| state.set_parent(child.id, Parent::Prim(parent.id)))??;
    Ok(())
}

pub async fn remove_child(api: &Api, self_rep: u32, child_rep: u32) -> anyhow::Result<()> {
    let (parent, child) = pair(api, self_rep, child_rep).await?;
    ensure_writable(api, &parent)?;
    if child.is_proxy {
        bail!("cannot remove proxy prim as child")
    }
    child.with(|state| state.set_parent(child.id, Parent::Root))??;
    Ok(())
}

pub async fn name(api: &Api, rep: u32) -> anyhow::Result<Option<String>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    Ok(prim.read_attr::<NameAttr>()?.map(|n| n.0))
}

pub async fn set_name(api: &Api, rep: u32, value: Option<String>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    match value {
        Some(s) => {
            anyhow::ensure!(s.len() <= MAX_NAME_BYTES, "name too long");
            prim.write_attr(&NameAttr(s))
        }
        None => prim.clear(NameAttr::KEY),
    }
}

pub async fn prefab(api: &Api, rep: u32) -> anyhow::Result<Option<Vec<u8>>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    prim.slot(slots::PREFAB)
}

pub async fn set_prefab(api: &Api, rep: u32, value: Option<Vec<u8>>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    prim.set_slot(slots::PREFAB, value)
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
    prim.read_attr::<XformAttr>()
}

pub async fn set_xform(api: &Api, rep: u32, value: Option<XformAttr>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    match value {
        Some(x) => {
            prim.write_attr(&x)?;
            if let Some(v) = NODE_TRANSFORM_REGISTRY.write().get_mut(&AbsoluteNodeId {
                doc:  prim.doc_id,
                node: prim.id,
            }) {
                v.local.translation = Vec3::from_array(x.translation);
                v.local.rotation = Quat::from_array(x.rotation);
                v.local.scale = Vec3::from_array(x.scale);
            }
            Ok(())
        }
        None => prim.clear(XformAttr::KEY),
    }
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
        let mut local = Affine3A::IDENTITY;
        let mut cur = Some(prim.id);
        while let Some(id) = cur {
            let t = NODE_TRANSFORM_REGISTRY
                .read()
                .get(&node(id))
                .map(|s| s.local)
                .unwrap_or_default();
            local = t.compute_affine() * local;
            cur = prim.with(|state| state.parent(id))?;
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
    Ok(prim
        .read_attr::<GravityScaleAttr>()?
        .map_or(1.0, |g| g.scale as f32))
}

pub async fn set_gravity_scale(api: &Api, rep: u32, value: f32) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    prim.write_attr(&GravityScaleAttr {
        scale: f64::from(value),
    })
}

pub async fn mesh(api: &Api, rep: u32) -> anyhow::Result<Option<PrimMesh>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    Ok(prim.read_attr::<MeshAttr>()?.map(|attr| PrimMesh {
        topology: topology_to_prim(attr.topology),
    }))
}

pub async fn set_mesh(api: &Api, rep: u32, value: Option<PrimMesh>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    prim.write_or_clear(value.map(|m| MeshAttr {
        topology: topology_from_prim(m.topology),
    }))
}

/// Writes a buffer to the prim's slot entry. Attribute and buffers are
/// separate entries, so writing one never rewrites the other.
fn set_buffer(api: &Api, prim: &PrimRes, slot: &str, bytes: Option<Vec<u8>>) -> anyhow::Result<()> {
    if bytes.is_some() {
        api.quota.spend(Flow::BlobUpload, 1.0)?;
    }
    prim.set_slot(slot, bytes)
}

/// The attribute and its buffers are separate entries, and a prim renders only
/// when it has both. Writing a buffer implies the attribute, so a caller that
/// wants the default topology never has to call `set-mesh` first.
fn ensure_mesh_attr(prim: &PrimRes) -> anyhow::Result<()> {
    if prim.read_attr::<MeshAttr>()?.is_some() {
        return Ok(());
    }
    prim.write_attr(&MeshAttr::default())
}

pub async fn set_mesh_stream(
    api: &Api,
    rep: u32,
    key: String,
    values: Option<Vec<f32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    anyhow::ensure!(key.len() <= MAX_NAME_BYTES, "mesh attribute key too long");
    let bytes = match values {
        Some(v) => {
            anyhow::ensure!(v.len() <= MAX_MESH_ELEMENTS, "mesh stream too large");
            ensure_mesh_attr(&prim)?;
            Some(f32s_to_bytes(&v))
        }
        None => None,
    };
    set_buffer(api, &prim, &slots::mesh_attribute(&key), bytes)
}

pub async fn set_mesh_indices_u32(
    api: &Api,
    rep: u32,
    values: Option<Vec<u32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let bytes = match values {
        Some(v) => {
            anyhow::ensure!(v.len() <= MAX_MESH_ELEMENTS, "mesh indices too large");
            ensure_mesh_attr(&prim)?;
            Some(u32s_to_bytes(&v))
        }
        None => None,
    };
    set_buffer(api, &prim, slots::MESH_INDICES, bytes)
}

pub async fn set_collider_vertices(
    api: &Api,
    rep: u32,
    values: Option<Vec<f32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let bytes = match values {
        Some(v) => {
            anyhow::ensure!(v.len() <= MAX_MESH_ELEMENTS, "collider vertices too large");
            Some(f32s_to_bytes(&v))
        }
        None => None,
    };
    set_buffer(api, &prim, slots::COLLIDER_VERTICES, bytes)
}

pub async fn set_collider_indices(
    api: &Api,
    rep: u32,
    values: Option<Vec<u32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    let bytes = match values {
        Some(v) => {
            anyhow::ensure!(v.len() <= MAX_MESH_ELEMENTS, "collider indices too large");
            Some(u32s_to_bytes(&v))
        }
        None => None,
    };
    set_buffer(api, &prim, slots::COLLIDER_INDICES, bytes)
}

pub async fn set_image_data(api: &Api, rep: u32, bytes: Option<Vec<u8>>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    set_buffer(api, &prim, slots::IMAGE_DATA, bytes)
}

const fn topology_to_prim(t: Topology) -> PrimTopology {
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
    Ok(prim.read_attr::<MaterialAttr>()?.map(material_attr_to_prim))
}

pub async fn set_material(api: &Api, rep: u32, value: Option<PrimMaterial>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    prim.write_or_clear(value.map(prim_to_material_attr))
}

pub async fn graph_overrides(api: &Api, rep: u32) -> anyhow::Result<Vec<(u16, PrimGraphValue)>> {
    let prim = get_prim(api, rep).await?;
    Ok(prim
        .read_attr::<GraphOverridesAttr>()?
        .map(|attr| {
            attr.overrides
                .into_iter()
                .map(|(index, value)| (index, graph_value_to_prim(value)))
                .collect()
        })
        .unwrap_or_default())
}

/// Clearing every override removes the attribute rather than writing an
/// empty map.
///
/// A prim using the graph's own defaults then carries no property at all,
/// which is the same shape `hsd-cli` compiles to.
pub async fn set_graph_overrides(
    api: &Api,
    rep: u32,
    values: Vec<(u16, PrimGraphValue)>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    anyhow::ensure!(
        values.len() <= MAX_PUBLIC_INPUTS,
        "a graph has at most {MAX_PUBLIC_INPUTS} public inputs"
    );

    let overrides = values
        .into_iter()
        .map(|(index, value)| {
            let value = prim_to_graph_value(value);
            anyhow::ensure!(is_finite(value), "override {index} is not finite");
            Ok((index, value))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;

    prim.write_or_clear((!overrides.is_empty()).then_some(GraphOverridesAttr { overrides }))
}

const fn graph_value_to_prim(value: GraphValue) -> PrimGraphValue {
    match value {
        GraphValue::Float(v) => PrimGraphValue::Float(v),
        GraphValue::Vec2(v) => PrimGraphValue::Vec2(v),
        GraphValue::Vec3(v) => PrimGraphValue::Vec3(v),
        GraphValue::Color([r, g, b, a]) => PrimGraphValue::Color(PrimColor { r, g, b, a }),
    }
}

const fn prim_to_graph_value(value: PrimGraphValue) -> GraphValue {
    match value {
        PrimGraphValue::Float(v) => GraphValue::Float(v),
        PrimGraphValue::Vec2(v) => GraphValue::Vec2(v),
        PrimGraphValue::Vec3(v) => GraphValue::Vec3(v),
        PrimGraphValue::Color(c) => GraphValue::Color([c.r, c.g, c.b, c.a]),
    }
}

fn material_attr_to_prim(attr: MaterialAttr) -> PrimMaterial {
    PrimMaterial {
        alpha_cutoff: attr.alpha_cutoff.map(|v| v as f32),
        alpha_mode:   attr.alpha_mode.and_then(|s| match s.as_str() {
            "add" => Some(PrimAlphaMode::Add),
            "blend" => Some(PrimAlphaMode::Blend),
            "mask" => Some(PrimAlphaMode::Mask),
            "multiply" => Some(PrimAlphaMode::Multiply),
            "opaque" => Some(PrimAlphaMode::Opaque),
            "premultiplied" => Some(PrimAlphaMode::PreMultiplied),
            _ => None,
        }),
        base_color:   attr.base_color.map(color_vec_to_prim),
        double_sided: attr.double_sided,
        emissive:     attr.emissive.map(color_vec_to_prim),
        metallic:     attr.metallic.map(|v| v as f32),
        roughness:    attr.roughness.map(|v| v as f32),
    }
}

fn prim_to_material_attr(m: PrimMaterial) -> MaterialAttr {
    MaterialAttr {
        alpha_cutoff: m.alpha_cutoff.map(f64::from),
        alpha_mode:   m.alpha_mode.map(|mode| {
            match mode {
                PrimAlphaMode::Add => "add",
                PrimAlphaMode::Blend => "blend",
                PrimAlphaMode::Mask => "mask",
                PrimAlphaMode::Multiply => "multiply",
                PrimAlphaMode::Opaque => "opaque",
                PrimAlphaMode::PreMultiplied => "premultiplied",
            }
            .to_string()
        }),
        base_color:   m.base_color.map(prim_color_to_vec),
        double_sided: m.double_sided,
        emissive:     m.emissive.map(prim_color_to_vec),
        metallic:     m.metallic.map(f64::from),
        roughness:    m.roughness.map(f64::from),
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
    Ok(prim.read_attr::<ImageAttr>()?.map(|attr| PrimImage {
        address_mode_u: attr.address_mode_u.map(|v| v as i32),
        address_mode_v: attr.address_mode_v.map(|v| v as i32),
        address_mode_w: attr.address_mode_w.map(|v| v as i32),
        mag_filter:     attr.mag_filter.map(|v| v as i32),
        min_filter:     attr.min_filter.map(|v| v as i32),
        mipmap_filter:  attr.mipmap_filter.map(|v| v as i32),
        srgb:           attr.srgb,
    }))
}

pub async fn set_image(api: &Api, rep: u32, value: Option<PrimImage>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    prim.write_or_clear(value.map(|img| ImageAttr {
        address_mode_u: img.address_mode_u.map(i64::from),
        address_mode_v: img.address_mode_v.map(i64::from),
        address_mode_w: img.address_mode_w.map(i64::from),
        mag_filter:     img.mag_filter.map(i64::from),
        min_filter:     img.min_filter.map(i64::from),
        mipmap_filter:  img.mipmap_filter.map(i64::from),
        srgb:           img.srgb,
    }))
}

pub async fn collider(api: &Api, rep: u32) -> anyhow::Result<Option<PrimCollider>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    Ok(prim.read_attr::<ColliderAttr>()?.map(|c| match c {
        ColliderAttr::Capsule { height, radius } => PrimCollider::Capsule {
            height: height as f32,
            radius: radius as f32,
        },
        ColliderAttr::ConvexHull => PrimCollider::ConvexHull,
        ColliderAttr::Cuboid { x, y, z } => PrimCollider::Cuboid([x as f32, y as f32, z as f32]),
        ColliderAttr::Cylinder { height, radius } => PrimCollider::Cylinder {
            height: height as f32,
            radius: radius as f32,
        },
        ColliderAttr::Sphere(r) => PrimCollider::Sphere(r as f32),
        ColliderAttr::Trimesh => PrimCollider::Trimesh,
    }))
}

pub async fn set_collider(api: &Api, rep: u32, value: Option<PrimCollider>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    prim.write_or_clear(value.map(|c| match c {
        PrimCollider::Capsule { height, radius } => ColliderAttr::Capsule {
            height: f64::from(height),
            radius: f64::from(radius),
        },
        PrimCollider::ConvexHull => ColliderAttr::ConvexHull,
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
        PrimCollider::Trimesh => ColliderAttr::Trimesh,
    }))
}

pub async fn rigid_body(api: &Api, rep: u32) -> anyhow::Result<Option<PrimRigidBody>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    Ok(prim
        .read_attr::<RigidBodyAttr>()?
        .map(rigid_body_attr_to_prim))
}

pub async fn set_rigid_body(
    api: &Api,
    rep: u32,
    value: Option<PrimRigidBody>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    prim.write_or_clear(value.map(prim_to_rigid_body_attr))
}

fn rigid_body_attr_to_prim(attr: RigidBodyAttr) -> PrimRigidBody {
    PrimRigidBody {
        kind:            match attr.kind.unwrap_or(RigidBodyKind::Dynamic) {
            RigidBodyKind::Dynamic => PrimRigidBodyKind::Dynamic,
            RigidBodyKind::Kinematic => PrimRigidBodyKind::Kinematic,
            RigidBodyKind::Static => PrimRigidBodyKind::Static,
        },
        angular_damping: attr.angular_damping.map(|v| v as f32),
        friction:        attr.friction.map(|v| v as f32),
        linear_damping:  attr.linear_damping.map(|v| v as f32),
        mass:            attr.mass.map(|v| v as f32),
        restitution:     attr.restitution.map(|v| v as f32),
    }
}

fn prim_to_rigid_body_attr(rb: PrimRigidBody) -> RigidBodyAttr {
    RigidBodyAttr {
        kind:            Some(match rb.kind {
            PrimRigidBodyKind::Dynamic => RigidBodyKind::Dynamic,
            PrimRigidBodyKind::Kinematic => RigidBodyKind::Kinematic,
            PrimRigidBodyKind::Static => RigidBodyKind::Static,
        }),
        angular_damping: rb.angular_damping.map(f64::from),
        friction:        rb.friction.map(f64::from),
        linear_damping:  rb.linear_damping.map(f64::from),
        mass:            rb.mass.map(f64::from),
        restitution:     rb.restitution.map(f64::from),
    }
}

pub async fn portal(api: &Api, rep: u32) -> anyhow::Result<Option<PrimPortal>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    Ok(prim.read_attr::<PortalAttr>()?.map(portal_attr_to_prim))
}

pub async fn set_portal(api: &Api, rep: u32, value: Option<PrimPortal>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    match value {
        Some(p) => prim.write_attr(&prim_portal_to_attr(p)?),
        None => prim.clear(PortalAttr::KEY),
    }
}

fn prim_portal_to_attr(p: PrimPortal) -> anyhow::Result<PortalAttr> {
    let destination = match p.destination {
        Some(d) => {
            let receptor = match d.receptor {
                Some(r) => Some(PortalReceptor {
                    document: DocId(r.document),
                    prim:     r
                        .prim
                        .parse::<PrimId>()
                        .map_err(|err| anyhow::anyhow!("invalid receptor prim id: {err}"))?,
                }),
                None => None,
            };
            Some(PortalDestination {
                receptor,
                space: d.space,
            })
        }
        None => None,
    };
    Ok(PortalAttr {
        destination,
        size_x: f64::from(p.size_x),
        size_y: f64::from(p.size_y),
    })
}

fn portal_attr_to_prim(attr: PortalAttr) -> PrimPortal {
    PrimPortal {
        destination: attr.destination.map(|d| PrimPortalDestination {
            receptor: d.receptor.map(|r| PrimPortalReceptor {
                document: r.document.0,
                prim:     r.prim.to_string(),
            }),
            space:    d.space,
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
    Ok(prim.read_attr::<SpawnAttr>()?.map(|a| PrimSpawn {
        radius: a.radius as f32,
    }))
}

pub async fn set_spawn(api: &Api, rep: u32, value: Option<PrimSpawn>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(api, &prim)?;
    prim.write_or_clear(value.map(|s| SpawnAttr {
        radius: f64::from(s.radius),
    }))
}

pub async fn relationships(api: &Api, rep: u32) -> anyhow::Result<Vec<(String, String)>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(Vec::new());
    }
    prim.with(|state| {
        state.get(prim.id).map_or_else(Vec::new, |p| {
            p.properties()
                .filter_map(|(name, value)| {
                    Some((name.to_string(), value.as_relationship()?.to_string()))
                })
                .collect()
        })
    })
}

pub async fn get_relationship(api: &Api, rep: u32, key: String) -> anyhow::Result<Option<String>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    prim.with(|state| state.relationship(prim.id, &key).map(|id| id.to_string()))
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
    match target {
        Some(target_id) => {
            let target = target_id
                .parse::<PrimId>()
                .map_err(|_| anyhow::anyhow!("invalid prim id: {target_id}"))?;
            prim.with(|state| {
                anyhow::ensure!(
                    state.exists(target),
                    "relationship target does not exist in this document"
                );
                state
                    .set_property(prim.id, &key, Property::Relationship(target))
                    .map_err(Into::into)
            })?
        }
        None => prim.clear(&key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prim_res() -> PrimRes {
        let mut state = SceneState::new();
        let id = state.create_prim(None);
        PrimRes {
            state: Arc::new(Mutex::new(state)),
            doc_id: DocId([0; 32]),
            id,
            is_proxy: false,
        }
    }

    #[test]
    fn first_buffer_defaults_the_mesh_attr() {
        let prim = prim_res();
        ensure_mesh_attr(&prim).expect("ensure mesh attr");
        assert_eq!(
            prim.read_attr::<MeshAttr>().expect("read mesh attr"),
            Some(MeshAttr {
                topology: Topology::TriangleList,
            })
        );
    }

    #[test]
    fn an_authored_topology_survives_a_buffer_write() {
        let prim = prim_res();
        prim.write_attr(&MeshAttr {
            topology: Topology::LineList,
        })
        .expect("write mesh attr");
        ensure_mesh_attr(&prim).expect("ensure mesh attr");
        assert_eq!(
            prim.read_attr::<MeshAttr>().expect("read mesh attr"),
            Some(MeshAttr {
                topology: Topology::LineList,
            })
        );
    }
}
