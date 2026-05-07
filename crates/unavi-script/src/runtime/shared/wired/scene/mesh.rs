use std::sync::Arc;

use bevy::mesh::PrimitiveTopology;
use blake3::Hash;
use hsd::{HsdMesh, topology::HydratedTopology};
use loro::{LoroDoc, LoroMap};
use loro_surgeon::{Hydrate, Reconcile};
use smol_str::SmolStr;

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api,
        registry::firewall::validate_firewall,
        wired::scene::util::{bytes_to_f32s, bytes_to_u32s, f32s_to_bytes, u32s_to_bytes},
    },
};

#[derive(Clone)]
pub struct MeshRes {
    pub doc: Arc<LoroDoc>,
    pub doc_id: Hash,
    pub id: SmolStr,
}

#[derive(Clone, Copy, Default)]
pub enum MeshTopology {
    PointList,
    LineList,
    LineStrip,
    #[default]
    TriangleList,
    TriangleStrip,
}

pub enum MeshIndices {
    Half(Vec<u16>),
    Full(Vec<u32>),
}

impl From<HydratedTopology> for MeshTopology {
    fn from(h: HydratedTopology) -> Self {
        match h.0 {
            PrimitiveTopology::PointList => Self::PointList,
            PrimitiveTopology::LineList => Self::LineList,
            PrimitiveTopology::LineStrip => Self::LineStrip,
            PrimitiveTopology::TriangleList => Self::TriangleList,
            PrimitiveTopology::TriangleStrip => Self::TriangleStrip,
        }
    }
}

impl From<MeshTopology> for HydratedTopology {
    fn from(t: MeshTopology) -> Self {
        Self(match t {
            MeshTopology::PointList => PrimitiveTopology::PointList,
            MeshTopology::LineList => PrimitiveTopology::LineList,
            MeshTopology::LineStrip => PrimitiveTopology::LineStrip,
            MeshTopology::TriangleList => PrimitiveTopology::TriangleList,
            MeshTopology::TriangleStrip => PrimitiveTopology::TriangleStrip,
        })
    }
}

pub fn clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .try_lock()?
        .meshes
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid mesh"))
}

pub fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.try_lock()?.meshes.remove(rep);
    Ok(())
}

fn get_mesh(api: &Api, rep: u32) -> anyhow::Result<MeshRes> {
    api.wired_scene
        .try_lock()?
        .meshes
        .get(rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid mesh rep: {rep}"))
}

fn mesh_map(doc: &LoroDoc, id: &str) -> anyhow::Result<LoroMap> {
    doc.get_map("hsd")
        .get_or_create_container("meshes", LoroMap::new())?
        .get_or_create_container(id, LoroMap::new())
        .map_err(Into::into)
}

fn hydrate_mesh(map: &LoroMap) -> HsdMesh {
    HsdMesh::hydrate(&map.get_deep_value()).unwrap_or_default()
}

pub fn id(api: &Api, rep: u32) -> anyhow::Result<String> {
    Ok(get_mesh(api, rep)?.id.to_string())
}

pub fn name(api: &Api, rep: u32) -> anyhow::Result<Option<String>> {
    let mesh = get_mesh(api, rep)?;
    let map = mesh_map(&mesh.doc, &mesh.id)?;
    Ok(hydrate_mesh(&map).name.map(|s| s.to_string()))
}

pub fn set_name(api: &Api, rep: u32, value: Option<String>) -> anyhow::Result<()> {
    let mesh = get_mesh(api, rep)?;
    validate_firewall(&api.doc_id, &mesh.doc_id, Channel::SceneWrite)?;
    let map = mesh_map(&mesh.doc, &mesh.id)?;
    let mut data = hydrate_mesh(&map);
    data.name = value.map(SmolStr::from);
    data.reconcile(&map)?;
    Ok(())
}

pub fn topology(api: &Api, rep: u32) -> anyhow::Result<MeshTopology> {
    let mesh = get_mesh(api, rep)?;
    let map = mesh_map(&mesh.doc, &mesh.id)?;
    Ok(hydrate_mesh(&map).topology.into())
}

pub fn set_topology(api: &Api, rep: u32, value: MeshTopology) -> anyhow::Result<()> {
    let mesh = get_mesh(api, rep)?;
    validate_firewall(&api.doc_id, &mesh.doc_id, Channel::SceneWrite)?;
    let map = mesh_map(&mesh.doc, &mesh.id)?;
    HydratedTopology::from(value).reconcile_field(&map, "topology")?;
    Ok(())
}

pub async fn indices(api: &Api, rep: u32) -> anyhow::Result<Option<MeshIndices>> {
    let mesh = get_mesh(api, rep)?;
    let map = mesh_map(&mesh.doc, &mesh.id)?;
    let Some(hash) = hydrate_mesh(&map).indices else {
        return Ok(None);
    };
    let bytes = super::fetch_blob(hash.into()).await?;
    Ok(Some(MeshIndices::Full(bytes_to_u32s(&bytes))))
}

pub async fn set_indices(api: &Api, rep: u32, value: Option<MeshIndices>) -> anyhow::Result<()> {
    let mesh = get_mesh(api, rep)?;
    validate_firewall(&api.doc_id, &mesh.doc_id, Channel::SceneWrite)?;
    let map = mesh_map(&mesh.doc, &mesh.id)?;
    let mut data = hydrate_mesh(&map);
    data.indices = match value {
        None => None,
        Some(MeshIndices::Half(v)) => {
            let u32s: Vec<u32> = v.into_iter().map(u32::from).collect();
            let hash = super::upload_blob(u32s_to_bytes(&u32s)).await?;
            Some(hash.into())
        }
        Some(MeshIndices::Full(v)) => {
            let hash = super::upload_blob(u32s_to_bytes(&v)).await?;
            Some(hash.into())
        }
    };
    data.reconcile(&map)?;
    Ok(())
}

async fn get_attribute(api: &Api, rep: u32, key: &str) -> anyhow::Result<Option<Vec<f32>>> {
    let mesh = get_mesh(api, rep)?;
    let map = mesh_map(&mesh.doc, &mesh.id)?;
    let Some(hash) = hydrate_mesh(&map).attributes.get(key).copied() else {
        return Ok(None);
    };
    let bytes = super::fetch_blob(hash.into()).await?;
    Ok(Some(bytes_to_f32s(&bytes)))
}

async fn set_attribute(
    api: &Api,
    rep: u32,
    key: &str,
    values: Option<Vec<f32>>,
) -> anyhow::Result<()> {
    let mesh = get_mesh(api, rep)?;
    validate_firewall(&api.doc_id, &mesh.doc_id, Channel::SceneWrite)?;
    let map = mesh_map(&mesh.doc, &mesh.id)?;
    let mut data = hydrate_mesh(&map);
    match values {
        None => {
            data.attributes.remove(key);
        }
        Some(v) => {
            let hash = super::upload_blob(f32s_to_bytes(&v)).await?;
            data.attributes.insert(SmolStr::from(key), hash.into());
        }
    }
    data.reconcile(&map)?;
    Ok(())
}

pub async fn positions(api: &Api, rep: u32) -> anyhow::Result<Option<Vec<f32>>> {
    get_attribute(api, rep, "POSITION").await
}

pub async fn set_positions(api: &Api, rep: u32, values: Option<Vec<f32>>) -> anyhow::Result<()> {
    set_attribute(api, rep, "POSITION", values).await
}

pub async fn normals(api: &Api, rep: u32) -> anyhow::Result<Option<Vec<f32>>> {
    get_attribute(api, rep, "NORMAL").await
}

pub async fn set_normals(api: &Api, rep: u32, values: Option<Vec<f32>>) -> anyhow::Result<()> {
    set_attribute(api, rep, "NORMAL", values).await
}

pub async fn tangents(api: &Api, rep: u32) -> anyhow::Result<Option<Vec<f32>>> {
    get_attribute(api, rep, "TANGENT").await
}

pub async fn set_tangents(api: &Api, rep: u32, values: Option<Vec<f32>>) -> anyhow::Result<()> {
    set_attribute(api, rep, "TANGENT", values).await
}

pub async fn colors(api: &Api, rep: u32) -> anyhow::Result<Option<Vec<f32>>> {
    get_attribute(api, rep, "COLOR").await
}

pub async fn set_colors(api: &Api, rep: u32, values: Option<Vec<f32>>) -> anyhow::Result<()> {
    set_attribute(api, rep, "COLOR", values).await
}

pub async fn uv0(api: &Api, rep: u32) -> anyhow::Result<Option<Vec<f32>>> {
    get_attribute(api, rep, "UV_0").await
}

pub async fn set_uv0(api: &Api, rep: u32, values: Option<Vec<f32>>) -> anyhow::Result<()> {
    set_attribute(api, rep, "UV_0", values).await
}

pub async fn uv1(api: &Api, rep: u32) -> anyhow::Result<Option<Vec<f32>>> {
    get_attribute(api, rep, "UV_1").await
}

pub async fn set_uv1(api: &Api, rep: u32, values: Option<Vec<f32>>) -> anyhow::Result<()> {
    set_attribute(api, rep, "UV_1", values).await
}
