use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobRequest, BlobResponse};
use bytemuck::{Pod, PodCastError, try_cast_slice};
use bytes::Bytes;

use crate::stage::Attrs;

pub fn parse_mesh_attrs(attrs: &Attrs, node: Entity, commands: &mut Commands) {
    if let Some(points) = attrs.mesh_points
        && let Some(indices) = attrs.mesh_indices
        && let Some(topology) = attrs.mesh_topology
    {
        let points = commands
            .spawn((BlobDep { target: node }, BlobRequest(points.0)))
            .id();
        let indices = commands
            .spawn((BlobDep { target: node }, BlobRequest(indices.0)))
            .id();

        let colors = attrs.mesh_colors.map(|hash| {
            commands
                .spawn((BlobDep { target: node }, BlobRequest(hash.0)))
                .id()
        });

        let normals = attrs.mesh_normals.map(|hash| {
            commands
                .spawn((BlobDep { target: node }, BlobRequest(hash.0)))
                .id()
        });

        let tangents = attrs.mesh_tangents.map(|hash| {
            commands
                .spawn((BlobDep { target: node }, BlobRequest(hash.0)))
                .id()
        });

        let uv_0 = attrs.mesh_uv_0.map(|hash| {
            commands
                .spawn((BlobDep { target: node }, BlobRequest(hash.0)))
                .id()
        });
        let uv_1 = attrs.mesh_uv_1.map(|hash| {
            commands
                .spawn((BlobDep { target: node }, BlobRequest(hash.0)))
                .id()
        });

        commands.entity(node).insert(MeshParams {
            topology: topology.0,
            points,
            indices,
            colors,
            normals,
            tangents,
            uv_0,
            uv_1,
        });
    } else {
        commands.entity(node).remove::<Mesh3d>();
    }
}

macro_rules! insert_mesh_attr {
    (
        $blobs:expr,
        $mesh:expr,
        $params:expr,
        $param:ident,
        $attr:expr,
        $values:path,
    ) => {
        if let Some(param) = $params.$param
            && let Ok(Some(bytes)) = $blobs.get_mut(param).map(|mut b| b.0.take())
        {
            match bytes_to_vec(&bytes) {
                Ok(v) => $mesh.insert_attribute($attr, $values(v)),
                Err(err) => warn!(?err, concat!("invalid ", stringify!($param), " buffer")),
            };
        }
    };
}

#[derive(Component)]
#[require(BlobDeps)]
pub struct MeshParams {
    topology: PrimitiveTopology,
    points: Entity,
    indices: Entity,
    colors: Option<Entity>,
    normals: Option<Entity>,
    tangents: Option<Entity>,
    uv_0: Option<Entity>,
    uv_1: Option<Entity>,
}

pub fn compile_meshes(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    loaded: Query<(Entity, &MeshParams), Added<BlobDepsLoaded>>,
    mut blobs: Query<&mut BlobResponse>,
) {
    for (ent, params) in loaded {
        let mut mesh = Mesh::new(params.topology, RenderAssetUsages::all());

        let Ok(Some(indices)) = blobs.get_mut(params.indices).map(|mut b| b.0.take()) else {
            continue;
        };
        let indices = match bytes_to_vec(&indices) {
            Ok(s) => s,
            Err(err) => {
                warn!(?err, "invalid indices");
                continue;
            }
        };
        mesh.insert_indices(Indices::U32(indices));

        let Ok(Some(points)) = blobs.get_mut(params.points).map(|mut b| b.0.take()) else {
            continue;
        };
        let points = match bytes_to_vec(&points) {
            Ok(s) => s,
            Err(err) => {
                warn!(?err, "invalid points");
                continue;
            }
        };
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(points),
        );

        insert_mesh_attr!(
            blobs,
            mesh,
            params,
            colors,
            Mesh::ATTRIBUTE_COLOR,
            VertexAttributeValues::Float32x4,
        );
        insert_mesh_attr!(
            blobs,
            mesh,
            params,
            normals,
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3,
        );
        insert_mesh_attr!(
            blobs,
            mesh,
            params,
            tangents,
            Mesh::ATTRIBUTE_TANGENT,
            VertexAttributeValues::Float32x4,
        );
        insert_mesh_attr!(
            blobs,
            mesh,
            params,
            uv_0,
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2,
        );
        insert_mesh_attr!(
            blobs,
            mesh,
            params,
            uv_1,
            Mesh::ATTRIBUTE_UV_1,
            VertexAttributeValues::Float32x2,
        );

        let handle = asset_server.add(mesh);

        commands
            .entity(ent)
            .insert(Mesh3d(handle))
            .insert(MeshMaterial3d(
                asset_server.add(StandardMaterial::default()),
            ));
    }
}

fn bytes_to_vec<T: Pod>(bytes: &Bytes) -> Result<Vec<T>, PodCastError> {
    let slice = try_cast_slice::<u8, T>(bytes)?;
    Ok(slice.to_vec())
}
