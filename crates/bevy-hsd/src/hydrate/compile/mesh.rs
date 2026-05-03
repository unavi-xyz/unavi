use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};
use bevy_wds::blob::{
    deps::{BlobDep, BlobDeps, BlobDepsLoaded},
    request::{BlobRequest, BlobResponse},
};
use bytemuck::{Pod, PodCastError, try_cast_slice};
use bytes::Bytes;
use smol_str::SmolStr;

use hsd::HsdMesh;

use crate::{DocRegistryMap, HsdChild, MeshId};

#[derive(Component)]
pub struct CompiledMesh(pub Handle<Mesh>);

/// Inline mesh geometry for testing or programmatic mesh creation.
#[derive(Clone, Default)]
pub struct MeshState {
    pub name: Option<String>,
    pub topology: PrimitiveTopology,
    pub indices: Option<Vec<u32>>,
    pub positions: Option<Vec<f32>>,
    pub normals: Option<Vec<f32>>,
    pub tangents: Option<Vec<f32>>,
    pub colors: Option<Vec<f32>>,
    pub uv0: Option<Vec<f32>>,
    pub uv1: Option<Vec<f32>>,
}

pub enum MeshGeometrySource {
    Hsd(Box<HsdMesh>),
    Inline(Box<MeshState>),
}

#[derive(Event)]
pub struct HsdMeshDespawned {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
}

#[derive(Event)]
pub struct HsdMeshGeometrySet {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
    pub source: MeshGeometrySource,
}

#[derive(Event)]
pub struct HsdMeshSpawned {
    pub doc_id: blake3::Hash,
    pub id: SmolStr,
}

#[derive(Component)]
pub struct MeshAttrName(pub SmolStr);

#[derive(Component)]
#[require(BlobDeps)]
pub struct MeshParams {
    pub topology: PrimitiveTopology,
    pub attr_deps: Vec<Entity>,
    pub indices: Option<Entity>,
}

pub(crate) fn handle_hsd_mesh_spawned(
    trigger: On<HsdMeshSpawned>,
    registry_map: Res<DocRegistryMap>,
    mut entity_maps: Query<&mut crate::HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "mesh spawned");
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(mut maps) = entity_maps.get_mut(doc_ent) else {
        return;
    };
    if maps.meshes.contains_key(&ev.id) {
        return;
    }
    let ent = commands
        .spawn((HsdChild(doc_ent), MeshId(ev.id.clone())))
        .id();
    maps.meshes.insert(ev.id.clone(), ent);
}

pub(crate) fn handle_hsd_mesh_despawned(
    trigger: On<HsdMeshDespawned>,
    registry_map: Res<DocRegistryMap>,
    mut entity_maps: Query<&mut crate::HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "mesh despawned");
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(mut maps) = entity_maps.get_mut(doc_ent) else {
        return;
    };
    let Some(ent) = maps.meshes.remove(&ev.id) else {
        return;
    };
    if let Ok(mut entity_cmd) = commands.get_entity(ent) {
        entity_cmd.despawn();
    }
}

pub(crate) fn handle_hsd_mesh_geometry_set(
    trigger: On<HsdMeshGeometrySet>,
    registry_map: Res<DocRegistryMap>,
    entity_maps: Query<&crate::HsdEntityMaps>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "mesh geometry set");
    let Some(doc_ent) = registry_map.get_entity(&ev.doc_id) else {
        return;
    };
    let Ok(maps) = entity_maps.get(doc_ent) else {
        return;
    };
    let Some(&ent) = maps.meshes.get(&ev.id) else {
        return;
    };
    commands
        .entity(ent)
        .try_remove::<BlobDepsLoaded>()
        .try_remove::<BlobDeps>()
        .try_remove::<MeshParams>();
    match &ev.source {
        MeshGeometrySource::Hsd(hsd_mesh) => {
            setup_hsd_mesh_blobs(ent, hsd_mesh, &mut commands);
        }
        MeshGeometrySource::Inline(state) => {
            attach_inline_mesh(ent, state, &mut commands);
        }
    }
}

fn setup_hsd_mesh_blobs(ent: Entity, mesh: &HsdMesh, commands: &mut Commands) {
    let mut attr_deps = Vec::new();

    for (name, hash) in &mesh.attributes {
        let dep = commands
            .spawn((
                BlobDep(ent),
                BlobRequest(hash.0),
                MeshAttrName(name.clone()),
            ))
            .id();
        attr_deps.push(dep);
    }

    let indices = mesh
        .indices
        .map(|hash| commands.spawn((BlobDep(ent), BlobRequest(hash.0))).id());

    commands.entity(ent).insert(MeshParams {
        topology: mesh.topology.0,
        attr_deps,
        indices,
    });
}

pub(crate) fn attach_inline_mesh(ent: Entity, state: &MeshState, commands: &mut Commands) {
    let mut attr_deps = Vec::new();

    macro_rules! add_attr {
        ($field:expr, $name:expr) => {
            if let Some(ref data) = $field {
                let bytes = Bytes::copy_from_slice(bytemuck::cast_slice::<f32, u8>(data));
                let dep = commands
                    .spawn((
                        BlobDep(ent),
                        MeshAttrName($name.into()),
                        BlobResponse(Some(bytes)),
                    ))
                    .id();
                attr_deps.push(dep);
            }
        };
    }

    add_attr!(state.positions, "POSITION");
    add_attr!(state.normals, "NORMAL");
    add_attr!(state.tangents, "TANGENT");
    add_attr!(state.colors, "COLOR");
    add_attr!(state.uv0, "UV_0");
    add_attr!(state.uv1, "UV_1");

    let indices = state.indices.as_ref().map(|idx| {
        let bytes = Bytes::copy_from_slice(bytemuck::cast_slice::<u32, u8>(idx));
        commands
            .spawn((BlobDep(ent), BlobResponse(Some(bytes))))
            .id()
    });

    commands.entity(ent).insert(MeshParams {
        topology: state.topology,
        attr_deps,
        indices,
    });
}

pub(crate) fn on_mesh_blobs_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    mesh_params: Query<(&MeshParams, Option<&CompiledMesh>)>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut blobs: Query<&mut BlobResponse>,
    attr_names: Query<&MeshAttrName>,
) {
    let ent = trigger.entity;
    let Ok((params, existing)) = mesh_params.get(ent) else {
        return;
    };

    let mut mesh = Mesh::new(params.topology, RenderAssetUsages::default());

    if let Some(idx_ent) = params.indices {
        let Ok(Some(bytes)) = blobs.get_mut(idx_ent).map(|mut b| b.0.take()) else {
            return;
        };
        let indices = match bytes_to_vec::<u32>(&bytes) {
            Ok(s) => s,
            Err(err) => {
                warn!(?err, "invalid indices");
                return;
            }
        };
        mesh.insert_indices(Indices::U32(indices));
    }

    for &dep_ent in &params.attr_deps {
        let Ok(name) = attr_names.get(dep_ent) else {
            warn!("attr name not found");
            continue;
        };
        let Ok(Some(bytes)) = blobs.get_mut(dep_ent).map(|mut b| b.0.take()) else {
            warn!("blob dep not found");
            continue;
        };

        let Some((attr, kind)) = mesh_attr_id(&name.0) else {
            continue;
        };

        match kind {
            MeshAttrKind::Float32x2 => match bytes_to_vec::<[f32; 2]>(&bytes) {
                Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x2(v)),
                Err(err) => warn!(?err, "invalid {} buffer", name.0),
            },
            MeshAttrKind::Float32x3 => match bytes_to_vec::<[f32; 3]>(&bytes) {
                Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x3(v)),
                Err(err) => warn!(?err, "invalid {} buffer", name.0),
            },
            MeshAttrKind::Float32x4 => match bytes_to_vec::<[f32; 4]>(&bytes) {
                Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x4(v)),
                Err(err) => warn!(?err, "invalid {} buffer", name.0),
            },
        }
    }

    debug!("compiled mesh {ent}");
    if let Some(CompiledMesh(handle)) = existing {
        if let Some(asset) = mesh_assets.get_mut(handle) {
            *asset = mesh;
            commands
                .entity(ent)
                .remove::<BlobDeps>()
                .remove::<BlobDepsLoaded>();
        }
    } else {
        let handle = mesh_assets.add(mesh);
        commands
            .entity(ent)
            .insert(CompiledMesh(handle))
            .remove::<BlobDeps>()
            .remove::<BlobDepsLoaded>();
    }
}

fn mesh_attr_id(name: &str) -> Option<(MeshVertexAttribute, MeshAttrKind)> {
    match name {
        "COLOR" => Some((Mesh::ATTRIBUTE_COLOR, MeshAttrKind::Float32x4)),
        "NORMAL" => Some((Mesh::ATTRIBUTE_NORMAL, MeshAttrKind::Float32x3)),
        "POSITION" => Some((Mesh::ATTRIBUTE_POSITION, MeshAttrKind::Float32x3)),
        "TANGENT" => Some((Mesh::ATTRIBUTE_TANGENT, MeshAttrKind::Float32x4)),
        "UV_0" => Some((Mesh::ATTRIBUTE_UV_0, MeshAttrKind::Float32x2)),
        "UV_1" => Some((Mesh::ATTRIBUTE_UV_1, MeshAttrKind::Float32x2)),
        _ => {
            warn!("unknown mesh attribute: {name}");
            None
        }
    }
}

enum MeshAttrKind {
    Float32x2,
    Float32x3,
    Float32x4,
}

fn bytes_to_vec<T: Pod>(bytes: &Bytes) -> Result<Vec<T>, PodCastError> {
    let slice = try_cast_slice::<u8, T>(bytes)?;
    Ok(slice.to_vec())
}
