use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobRequest, BlobResponse};
use bytemuck::{Pod, PodCastError, try_cast_slice};
use bytes::Bytes;
use smol_str::SmolStr;

use crate::{
    CompiledMesh, HsdChild,
    cache::{MeshState, SceneRegistry},
    data::HsdMesh,
};

pub enum MeshGeometrySource {
    Inline,
    Hsd(Box<HsdMesh>),
}

#[derive(Event)]
pub struct HsdMeshDespawned {
    pub doc: Entity,
    pub id: SmolStr,
}

#[derive(Event)]
pub struct HsdMeshGeometrySet {
    pub doc: Entity,
    pub id: SmolStr,
    pub source: MeshGeometrySource,
}

#[derive(Event)]
pub struct HsdMeshSpawned {
    pub doc: Entity,
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
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "mesh spawned");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let inner = registry
        .0
        .meshes
        .lock()
        .expect("meshes lock")
        .get(&ev.id)
        .cloned();
    let Some(inner) = inner else { return };
    if inner.entity.lock().expect("entity lock").is_some() {
        return;
    }
    let ent = commands.spawn(HsdChild { doc: ev.doc }).id();
    *inner.entity.lock().expect("entity lock") = Some(ent);
}

pub(crate) fn handle_hsd_mesh_despawned(
    trigger: On<HsdMeshDespawned>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "mesh despawned");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let inner = {
        let mut meshes = registry.0.meshes.lock().expect("meshes lock");
        meshes.remove(&ev.id)
    };
    let Some(inner) = inner else { return };
    let ent = *inner.entity.lock().expect("entity lock");
    if let Some(ent) = ent {
        commands.entity(ent).despawn();
    }
}

pub(crate) fn handle_hsd_mesh_geometry_set(
    trigger: On<HsdMeshGeometrySet>,
    registries: Query<&SceneRegistry>,
    mut commands: Commands,
) {
    let ev = trigger.event();
    debug!(id = %ev.id, "mesh geometry set");
    let Ok(registry) = registries.get(ev.doc) else {
        return;
    };
    let ent = registry
        .0
        .meshes
        .lock()
        .expect("meshes lock")
        .get(&ev.id)
        .and_then(|m| *m.entity.lock().expect("entity lock"));
    let Some(ent) = ent else { return };
    commands
        .entity(ent)
        .remove::<BlobDepsLoaded>()
        .remove::<BlobDeps>()
        .remove::<MeshParams>();
    match &ev.source {
        MeshGeometrySource::Hsd(hsd_mesh) => {
            setup_hsd_mesh_blobs(ent, hsd_mesh, &mut commands);
        }
        MeshGeometrySource::Inline => {
            let state = registry
                .0
                .meshes
                .lock()
                .expect("meshes lock")
                .get(&ev.id)
                .map(|m| m.state.lock().expect("mesh state lock").clone());
            let Some(state) = state else { return };
            attach_inline_mesh(ent, &state, &mut commands);
        }
    }
}

fn setup_hsd_mesh_blobs(ent: Entity, mesh: &HsdMesh, commands: &mut Commands) {
    let mut attr_deps = Vec::new();

    for (name, hash) in &mesh.attributes {
        let dep = commands
            .spawn((
                BlobDep { owner: ent },
                BlobRequest(hash.0),
                MeshAttrName(name.clone()),
            ))
            .id();
        attr_deps.push(dep);
    }

    let indices = mesh.indices.map(|hash| {
        commands
            .spawn((BlobDep { owner: ent }, BlobRequest(hash.0)))
            .id()
    });

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
                        BlobDep { owner: ent },
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
            .spawn((BlobDep { owner: ent }, BlobResponse(Some(bytes))))
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
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut blobs: Query<&mut BlobResponse>,
    attr_names: Query<&MeshAttrName>,
) {
    let ent = trigger.entity;
    let Ok((params, existing)) = mesh_params.get(ent) else {
        return;
    };

    let mut mesh = Mesh::new(params.topology, RenderAssetUsages::all());

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
        let handle = asset_server.add(mesh);
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
