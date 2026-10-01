use std::{
    collections::BTreeMap,
    mem::size_of_val,
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
    transform::components::Transform,
};
use hsd::{
    attributes::{
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
        material::{
            AlphaMode,
            ColorVec,
            MaterialAttr,
        },
        mesh::{
            self,
            MeshIndices,
            MeshStream,
            Topology,
        },
        name::NameAttr,
        parent::ParentAttr,
        portal::{
            PortalAttr,
            PortalDestination,
        },
        reference::ReferenceAttr,
        rigid_body::{
            RigidBodyAttr,
            RigidBodyKind,
        },
        shader::{
            MAX_PUBLIC_INPUTS,
            ShaderGraph,
            overrides::GraphOverridesAttr,
            validate::validate,
            value::{
                GraphValue,
                is_finite,
            },
        },
        spawn::SpawnAttr,
        text::{
            TextAlign,
            TextAnchor,
            TextAttr,
            TextBillboard,
        },
        xform::XformAttr,
    },
    bounds::{
        MAX_MESH_STREAM_BYTES,
        MAX_NAME_BYTES,
        MAX_TEXT_BYTES,
    },
    id::{
        DocId,
        PrimId,
    },
    property::{
        Payload,
        Property,
        name::PropName,
        value::Value,
    },
    state::HsdState,
};
use unavi_physics::finite;
use unavi_policy::quota::Flow;
use unavi_space::state::message::SessionWrite;

use crate::{
    error::ScriptError,
    runtime::shared::{
        Api,
        registry::transform::AbsoluteNodeId,
        wired::scene::util::{
            bytes_to_f32s,
            f32s_to_bytes,
            u32s_to_bytes,
        },
    },
};

#[derive(Clone)]
pub struct PrimRes {
    pub state:    Arc<Mutex<HsdState>>,
    pub doc_id:   DocId,
    pub id:       PrimId,
    /// Proxy prims are read-only from scripts.
    pub is_proxy: bool,
}

#[derive(Clone, Copy, Default)]
pub struct PrimColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Runtime mirror of the graph format's `GraphValue`.
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

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum PrimTextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum PrimTextAnchor {
    #[default]
    Baseline,
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum PrimTextBillboard {
    #[default]
    None,
    Yaw,
    Full,
}

#[derive(Default)]
pub struct PrimText {
    pub value:         String,
    pub size:          Option<f32>,
    pub align:         Option<PrimTextAlign>,
    pub anchor:        Option<PrimTextAnchor>,
    pub wrap:          Option<f32>,
    pub line_height:   Option<f32>,
    pub color:         Option<PrimColor>,
    pub outline:       Option<PrimColor>,
    pub outline_width: Option<f32>,
    pub emissive:      Option<f32>,
    pub billboard:     Option<PrimTextBillboard>,
}

pub struct PrimPortal {
    pub destination: Option<PortalDestination>,
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
    /// Writes are synchronous and land in in-memory state only; nothing here
    /// touches storage.
    fn with<T>(&self, f: impl FnOnce(&mut HsdState) -> T) -> anyhow::Result<T> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("scene state poisoned"))?;
        Ok(f(&mut state))
    }

    fn read_attr<A: Property>(&self) -> anyhow::Result<Option<A>> {
        self.with(|state| state.attribute::<A>(self.id).and_then(Result::ok))
    }

    fn write_attr<A: Property>(&self, value: &A) -> anyhow::Result<()> {
        self.with(|state| state.set_attribute(self.id, value))??;
        Ok(())
    }

    fn read_value<V: Payload>(&self, name: &PropName) -> anyhow::Result<Option<V>> {
        self.with(|state| state.payload::<V>(self.id, name).and_then(Result::ok))
    }

    fn write_value<V: Payload>(&self, name: &PropName, value: &V) -> anyhow::Result<()> {
        self.with(|state| state.set_payload(self.id, name, value))??;
        Ok(())
    }

    fn clear(&self, name: &PropName) -> anyhow::Result<()> {
        self.with(|state| state.remove_property(self.id, name))
    }

    /// Removes every field of `namespace` the prim resolves, so a relationship
    /// or texture binding that lives alongside a group's own field never
    /// outlives the group it was set on.
    fn clear_namespace(&self, namespace: &str) -> anyhow::Result<()> {
        self.with(|state| state.remove_group(self.id, namespace))
    }

    fn write_or_clear<A: Property>(&self, value: Option<A>) -> anyhow::Result<()> {
        value.map_or_else(|| self.clear(&A::NAME), |attr| self.write_attr(&attr))
    }

    /// Sets a collider buffer. A prim with no collider has nowhere to put it,
    /// so this errors rather than silently dropping it.
    fn set_collider_buffer<A: Property>(&self, value: Option<A>) -> anyhow::Result<()> {
        self.with(|state| -> anyhow::Result<()> {
            anyhow::ensure!(
                state.attribute::<ColliderKind>(self.id).is_some(),
                "prim has no collider to set buffers on"
            );
            match value {
                Some(v) => state.set_attribute(self.id, &v)?,
                None => state.remove_property(self.id, &A::NAME),
            }
            Ok(())
        })?
    }
}

fn ensure_writable(prim: &PrimRes) -> anyhow::Result<()> {
    if prim.is_proxy {
        bail!("cannot write proxy prim")
    }
    Ok(())
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
    ensure_writable(&parent)?;
    if child.is_proxy {
        bail!("cannot add proxy prim as child")
    }
    anyhow::ensure!(
        Arc::ptr_eq(&parent.state, &child.state),
        "prims must belong to the same document"
    );
    child.with(|state| state.set_parent(child.id, ParentAttr::Prim(parent.id)))??;
    Ok(())
}

pub async fn remove_child(api: &Api, self_rep: u32, child_rep: u32) -> anyhow::Result<()> {
    let (parent, child) = pair(api, self_rep, child_rep).await?;
    ensure_writable(&parent)?;
    if child.is_proxy {
        bail!("cannot remove proxy prim as child")
    }
    child.with(|state| state.set_parent(child.id, ParentAttr::Root))??;
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
    ensure_writable(&prim)?;
    match value {
        Some(s) => {
            anyhow::ensure!(s.len() <= MAX_NAME_BYTES, "name too long");
            prim.write_attr(&NameAttr(s))
        }
        None => prim.clear(&NameAttr::NAME),
    }
}

/// What present peers say about this prim this session.
///
/// Read from the session record rather than from the composed state: the
/// record is what replication and attribution are kept in, and what a peer
/// stated is not the same question as what the prim currently resolves to.
pub async fn session(api: &Api, rep: u32) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
    let prim = get_prim(api, rep).await?;
    let Some(space) = api.view.space_of(prim.doc_id) else {
        return Ok(Vec::new());
    };
    let replicas = api.view.replicas();
    Ok(replicas
        .session_keys(space, prim.doc_id, prim.id)
        .into_iter()
        .filter_map(|name| {
            replicas
                .session_value(space, prim.doc_id, prim.id, &name)
                .map(|value| (name.to_string(), value))
        })
        .collect())
}

/// States session opinions on this prim, as one atomic batch.
///
/// Every write carries one stamp and lands together, so a tick's dirty set
/// cannot render by halves. A document with no space has no session to speak
/// of and is refused.
pub async fn set_session(
    api: &Api,
    rep: u32,
    values: Vec<(String, Option<Vec<u8>>)>,
) -> anyhow::Result<Result<(), ScriptError>> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    let Some(space) = api.view.space_of(prim.doc_id) else {
        anyhow::bail!("document is not in a tracked space");
    };

    let writes = values
        .into_iter()
        .map(|(name, value)| SessionWrite {
            prim: prim.id,
            name,
            value,
        })
        .collect();

    Ok(api
        .view
        .set_session(space, prim.doc_id, writes)
        .await
        .map_err(Into::into))
}

pub async fn reference(api: &Api, rep: u32) -> anyhow::Result<Option<Vec<u8>>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    Ok(prim
        .read_attr::<ReferenceAttr>()?
        .map(|target| target.0.0.to_vec()))
}

pub async fn set_reference(api: &Api, rep: u32, value: Option<Vec<u8>>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    let target = value
        .map(|bytes| {
            <[u8; 32]>::try_from(bytes.as_slice())
                .map(|bytes| ReferenceAttr(DocId(bytes)))
                .map_err(|_| anyhow::anyhow!("document id must be 32 bytes"))
        })
        .transpose()?;
    prim.write_or_clear(target)
}

pub async fn xform(api: &Api, rep: u32) -> anyhow::Result<Option<XformAttr>> {
    let prim = get_prim(api, rep).await?;
    let local = api
        .transforms
        .node(&AbsoluteNodeId {
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
    ensure_writable(&prim)?;
    match value {
        Some(x) => {
            if finite::vec3(x.translation).is_none()
                || finite::quat(x.rotation).is_none()
                || finite::vec3(x.scale).is_none()
            {
                bail!("xform is not a transform: {x:?}");
            }
            prim.write_attr(&x)?;
            api.transforms.set_local(
                &AbsoluteNodeId {
                    doc:  prim.doc_id,
                    node: prim.id,
                },
                Transform {
                    translation: Vec3::from_array(x.translation),
                    rotation:    Quat::from_array(x.rotation),
                    scale:       Vec3::from_array(x.scale),
                },
            );
            Ok(())
        }
        None => prim.clear(&XformAttr::NAME),
    }
}

pub async fn global_xform(api: &Api, rep: u32) -> anyhow::Result<XformAttr> {
    let prim = get_prim(api, rep).await?;
    let world = if prim.is_proxy {
        api.transforms
            .node(&AbsoluteNodeId {
                doc:  prim.doc_id,
                node: prim.id,
            })
            .map_or(Affine3A::IDENTITY, |s| s.world.affine())
    } else {
        api.transforms.world_of(prim.doc_id, prim.id, |id| {
            prim.with(|state| state.parent(id))
        })?
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
    ensure_writable(&prim)?;
    prim.write_attr(&GravityScaleAttr {
        scale: f64::from(value),
    })
}

pub async fn mesh(api: &Api, rep: u32) -> anyhow::Result<Option<PrimMesh>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    Ok(prim.read_attr::<Topology>()?.map(|topology| PrimMesh {
        topology: topology_to_prim(topology),
    }))
}

pub async fn set_mesh(api: &Api, rep: u32, value: Option<PrimMesh>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    value.map_or_else(
        || prim.clear_namespace(Topology::NAME.group()),
        |mesh| prim.write_attr(&topology_from_prim(mesh.topology)),
    )
}

pub async fn set_mesh_stream(
    api: &Api,
    rep: u32,
    key: String,
    values: Option<Vec<f32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    let name = mesh::stream(&key)?;
    match values {
        Some(v) => {
            anyhow::ensure!(
                size_of_val(v.as_slice()) <= MAX_MESH_STREAM_BYTES,
                "mesh stream too large"
            );
            crate::quota::acquire(&api.quota, Flow::BlobUpload, 1.0).await?;
            prim.write_value(&name, &MeshStream(f32s_to_bytes(&v)))
        }
        None => prim.clear(&name),
    }
}

pub async fn set_mesh_indices_u32(
    api: &Api,
    rep: u32,
    values: Option<Vec<u32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    match values {
        Some(v) => {
            anyhow::ensure!(
                size_of_val(v.as_slice()) <= MAX_MESH_STREAM_BYTES,
                "mesh indices too large"
            );
            crate::quota::acquire(&api.quota, Flow::BlobUpload, 1.0).await?;
            prim.write_attr(&MeshIndices(u32s_to_bytes(&v)))
        }
        None => prim.clear(&MeshIndices::NAME),
    }
}

/// Reads a vertex stream back out of the prim's own `mesh/stream:<key>`
/// field. A proxy's streams are not present locally, so it reads as having
/// none.
pub async fn mesh_stream(api: &Api, rep: u32, key: String) -> anyhow::Result<Option<Vec<f32>>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    let name = mesh::stream(&key)?;
    Ok(prim
        .read_value::<MeshStream>(&name)?
        .map(|stream| bytes_to_f32s(&stream.0)))
}

pub async fn set_collider_vertices(
    api: &Api,
    rep: u32,
    values: Option<Vec<f32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    let value = match values {
        Some(v) => {
            anyhow::ensure!(
                size_of_val(v.as_slice()) <= MAX_MESH_STREAM_BYTES,
                "collider vertices too large"
            );
            crate::quota::acquire(&api.quota, Flow::BlobUpload, 1.0).await?;
            Some(ColliderVertices(f32s_to_bytes(&v)))
        }
        None => None,
    };
    prim.set_collider_buffer(value)
}

pub async fn set_collider_indices(
    api: &Api,
    rep: u32,
    values: Option<Vec<u32>>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    let value = match values {
        Some(v) => {
            anyhow::ensure!(
                size_of_val(v.as_slice()) <= MAX_MESH_STREAM_BYTES,
                "collider indices too large"
            );
            crate::quota::acquire(&api.quota, Flow::BlobUpload, 1.0).await?;
            Some(ColliderIndices(u32s_to_bytes(&v)))
        }
        None => None,
    };
    prim.set_collider_buffer(value)
}

pub async fn set_image_data(api: &Api, rep: u32, bytes: Option<Vec<u8>>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    match bytes {
        Some(b) => {
            crate::quota::acquire(&api.quota, Flow::BlobUpload, 1.0).await?;
            prim.write_attr(&ImageData(b))
        }
        None => prim.clear(&ImageData::NAME),
    }
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
    ensure_writable(&prim)?;
    value.map_or_else(
        || prim.clear_namespace(MaterialAttr::NAME.group()),
        |v| prim.write_attr(&prim_to_material_attr(v)),
    )
}

/// Validates a script-built graph and stores it as the prim's own shading.
///
/// A graph is bounded by its shape — fixed arity, no loop, no branch — so
/// building one is no more dangerous than building a mesh.
pub async fn set_material_graph(
    api: &Api,
    rep: u32,
    value: Option<ShaderGraph>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;

    match value {
        Some(graph) => {
            validate(&graph)?;
            crate::quota::acquire(&api.quota, Flow::BlobUpload, 1.0).await?;
            prim.write_attr(&graph)
        }
        // Clearing the graph drops its overrides and texture bindings too.
        None => prim.clear_namespace(ShaderGraph::NAME.group()),
    }
}

pub async fn graph_overrides(api: &Api, rep: u32) -> anyhow::Result<Vec<(u16, PrimGraphValue)>> {
    let prim = get_prim(api, rep).await?;
    Ok(prim
        .read_attr::<GraphOverridesAttr>()?
        .map_or_default(|attr| {
            attr.overrides
                .into_iter()
                .map(|(index, value)| (index, graph_value_to_prim(value)))
                .collect()
        }))
}

/// Clearing every override removes the attribute rather than writing an empty
/// map.
pub async fn set_graph_overrides(
    api: &Api,
    rep: u32,
    values: Vec<(u16, PrimGraphValue)>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
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
        alpha_mode:   attr.alpha_mode.map(|mode| match mode {
            AlphaMode::Add => PrimAlphaMode::Add,
            AlphaMode::Blend => PrimAlphaMode::Blend,
            AlphaMode::Mask => PrimAlphaMode::Mask,
            AlphaMode::Multiply => PrimAlphaMode::Multiply,
            AlphaMode::Opaque => PrimAlphaMode::Opaque,
            AlphaMode::Premultiplied => PrimAlphaMode::PreMultiplied,
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
        alpha_mode:   m.alpha_mode.map(|mode| match mode {
            PrimAlphaMode::Add => AlphaMode::Add,
            PrimAlphaMode::Blend => AlphaMode::Blend,
            PrimAlphaMode::Mask => AlphaMode::Mask,
            PrimAlphaMode::Multiply => AlphaMode::Multiply,
            PrimAlphaMode::Opaque => AlphaMode::Opaque,
            PrimAlphaMode::PreMultiplied => AlphaMode::Premultiplied,
        }),
        base_color:   m.base_color.map(prim_color_to_vec),
        double_sided: m.double_sided,
        emissive:     m.emissive.map(prim_color_to_vec),
        metallic:     m.metallic.map(f64::from),
        roughness:    m.roughness.map(f64::from),
    }
}

pub async fn text(api: &Api, rep: u32) -> anyhow::Result<Option<PrimText>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    Ok(prim.read_attr::<TextAttr>()?.map(text_attr_to_prim))
}

pub async fn set_text(api: &Api, rep: u32, value: Option<PrimText>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    if let Some(value) = &value {
        anyhow::ensure!(
            value.value.len() <= MAX_TEXT_BYTES,
            "text too long: {} bytes exceeds {MAX_TEXT_BYTES}",
            value.value.len()
        );
    }
    prim.write_or_clear(value.map(prim_to_text_attr))
}

fn text_attr_to_prim(attr: TextAttr) -> PrimText {
    PrimText {
        value:         attr.value,
        size:          attr.size.map(|v| v as f32),
        align:         attr.align.map(|align| match align {
            TextAlign::Left => PrimTextAlign::Left,
            TextAlign::Center => PrimTextAlign::Center,
            TextAlign::Right => PrimTextAlign::Right,
        }),
        anchor:        attr.anchor.map(|anchor| match anchor {
            TextAnchor::Baseline => PrimTextAnchor::Baseline,
            TextAnchor::Top => PrimTextAnchor::Top,
            TextAnchor::Middle => PrimTextAnchor::Middle,
            TextAnchor::Bottom => PrimTextAnchor::Bottom,
        }),
        wrap:          attr.wrap.map(|v| v as f32),
        line_height:   attr.line_height.map(|v| v as f32),
        color:         attr.color.map(color_vec_to_prim),
        outline:       attr.outline.map(color_vec_to_prim),
        outline_width: attr.outline_width.map(|v| v as f32),
        emissive:      attr.emissive.map(|v| v as f32),
        billboard:     attr.billboard.map(|billboard| match billboard {
            TextBillboard::None => PrimTextBillboard::None,
            TextBillboard::Yaw => PrimTextBillboard::Yaw,
            TextBillboard::Full => PrimTextBillboard::Full,
        }),
    }
}

fn prim_to_text_attr(t: PrimText) -> TextAttr {
    TextAttr {
        value:         t.value,
        size:          t.size.map(f64::from),
        align:         t.align.map(|align| match align {
            PrimTextAlign::Left => TextAlign::Left,
            PrimTextAlign::Center => TextAlign::Center,
            PrimTextAlign::Right => TextAlign::Right,
        }),
        anchor:        t.anchor.map(|anchor| match anchor {
            PrimTextAnchor::Baseline => TextAnchor::Baseline,
            PrimTextAnchor::Top => TextAnchor::Top,
            PrimTextAnchor::Middle => TextAnchor::Middle,
            PrimTextAnchor::Bottom => TextAnchor::Bottom,
        }),
        wrap:          t.wrap.map(f64::from),
        line_height:   t.line_height.map(f64::from),
        color:         t.color.map(prim_color_to_vec),
        outline:       t.outline.map(prim_color_to_vec),
        outline_width: t.outline_width.map(f64::from),
        emissive:      t.emissive.map(f64::from),
        billboard:     t.billboard.map(|billboard| match billboard {
            PrimTextBillboard::None => TextBillboard::None,
            PrimTextBillboard::Yaw => TextBillboard::Yaw,
            PrimTextBillboard::Full => TextBillboard::Full,
        }),
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

pub async fn image(api: &Api, rep: u32) -> anyhow::Result<Option<ImageSampler>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    prim.read_attr::<ImageSampler>()
}

pub async fn set_image(api: &Api, rep: u32, value: Option<ImageSampler>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    value.map_or_else(
        || prim.clear_namespace(ImageSampler::NAME.group()),
        |v| prim.write_attr(&v),
    )
}

pub async fn collider(api: &Api, rep: u32) -> anyhow::Result<Option<PrimCollider>> {
    let prim = get_prim(api, rep).await?;
    if prim.is_proxy {
        return Ok(None);
    }
    Ok(prim.read_attr::<ColliderKind>()?.map(|kind| match kind {
        ColliderKind::Capsule { height, radius } => PrimCollider::Capsule {
            height: height as f32,
            radius: radius as f32,
        },
        ColliderKind::ConvexHull => PrimCollider::ConvexHull,
        ColliderKind::Cuboid { x, y, z } => PrimCollider::Cuboid([x as f32, y as f32, z as f32]),
        ColliderKind::Cylinder { height, radius } => PrimCollider::Cylinder {
            height: height as f32,
            radius: radius as f32,
        },
        ColliderKind::Sphere(r) => PrimCollider::Sphere(r as f32),
        ColliderKind::Trimesh => PrimCollider::Trimesh,
    }))
}

pub async fn set_collider(api: &Api, rep: u32, value: Option<PrimCollider>) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    value.map_or_else(
        || prim.clear_namespace(ColliderKind::NAME.group()),
        |v| prim.write_attr(&prim_to_collider_kind(v)),
    )
}

fn prim_to_collider_kind(c: PrimCollider) -> ColliderKind {
    match c {
        PrimCollider::Capsule { height, radius } => ColliderKind::Capsule {
            height: f64::from(height),
            radius: f64::from(radius),
        },
        PrimCollider::ConvexHull => ColliderKind::ConvexHull,
        PrimCollider::Cuboid([x, y, z]) => ColliderKind::Cuboid {
            x: f64::from(x),
            y: f64::from(y),
            z: f64::from(z),
        },
        PrimCollider::Cylinder { height, radius } => ColliderKind::Cylinder {
            height: f64::from(height),
            radius: f64::from(radius),
        },
        PrimCollider::Sphere(r) => ColliderKind::Sphere(f64::from(r)),
        PrimCollider::Trimesh => ColliderKind::Trimesh,
    }
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
    ensure_writable(&prim)?;
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
    ensure_writable(&prim)?;
    value.map_or_else(
        || prim.clear(&PortalAttr::NAME),
        |p| prim.write_attr(&prim_portal_to_attr(p)),
    )
}

/// States `destination` on this prim's portal for the session, keeping the
/// portal's composed size.
///
/// A session opinion replaces the whole attribute, so the size rides along with
/// the destination every peer receives.
pub async fn set_session_destination(
    api: &Api,
    rep: u32,
    destination: PortalDestination,
) -> anyhow::Result<Result<(), ScriptError>> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    let Some(mut portal) = prim.read_attr::<PortalAttr>()? else {
        bail!("prim has no portal");
    };
    portal.destination = Some(destination);
    let value = portal.encode()?;
    let Some(space) = api.view.space_of(prim.doc_id) else {
        bail!("document is not in a tracked space");
    };

    Ok(api
        .view
        .set_session(
            space,
            prim.doc_id,
            vec![SessionWrite {
                prim:  prim.id,
                name:  PortalAttr::NAME.to_string(),
                value: Some(value),
            }],
        )
        .await
        .map_err(Into::into))
}

fn prim_portal_to_attr(p: PrimPortal) -> PortalAttr {
    PortalAttr {
        destination: p.destination,
        size_x:      f64::from(p.size_x),
        size_y:      f64::from(p.size_y),
    }
}

const fn portal_attr_to_prim(attr: PortalAttr) -> PrimPortal {
    PrimPortal {
        destination: attr.destination,
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
    ensure_writable(&prim)?;
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
    let name: PropName = key.parse()?;
    prim.with(|state| state.relationship(prim.id, &name).map(|id| id.to_string()))
}

pub async fn set_relationship(
    api: &Api,
    rep: u32,
    key: String,
    target: Option<String>,
) -> anyhow::Result<()> {
    let prim = get_prim(api, rep).await?;
    ensure_writable(&prim)?;
    let name: PropName = key.parse()?;
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
                    .set_property(prim.id, &name, Value::Relationship(target))
                    .map_err(Into::into)
            })?
        }
        // Only clears a field that currently holds a relationship: this door
        // cannot be used to delete an attribute such as `mesh/topology`.
        None => prim.with(|state| {
            if state.relationship(prim.id, &name).is_some() {
                state.remove_property(prim.id, &name);
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prim_res() -> PrimRes {
        let mut state = HsdState::new();
        let id = state.create_prim(None);
        PrimRes {
            state: Arc::new(Mutex::new(state)),
            doc_id: DocId([0; 32]),
            id,
            is_proxy: false,
        }
    }

    #[test]
    fn a_stream_write_leaves_topology_unset() {
        let prim = prim_res();
        let name = mesh::stream("POSITION").expect("valid stream name");
        prim.write_value(&name, &MeshStream(vec![0; 12]))
            .expect("write stream");
        assert!(
            prim.read_attr::<Topology>()
                .expect("read topology")
                .is_none(),
            "topology is its own field, independent of any stream"
        );
        assert!(
            prim.read_value::<MeshStream>(&name)
                .expect("read stream")
                .is_some()
        );
    }

    #[test]
    fn an_authored_topology_survives_a_stream_write() {
        let prim = prim_res();
        prim.write_attr(&Topology::LineList)
            .expect("write topology");
        let name = mesh::stream("POSITION").expect("valid stream name");
        prim.write_value(&name, &MeshStream(vec![0; 12]))
            .expect("write stream");
        let topology = prim
            .read_attr::<Topology>()
            .expect("read topology")
            .expect("present");
        assert_eq!(topology, Topology::LineList);
    }
}
