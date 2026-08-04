use bevy::{
    mesh::{
        Indices,
        VertexAttributeValues,
    },
    prelude::*,
};
use hsd::attributes::{
    mesh::{
        MeshAttr,
        Topology,
    },
    slots,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_mesh_lifecycle(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &MeshAttr {
            topology: Topology::TriangleList,
        },
    );
    ctx.set_bulk(
        root,
        &slots::mesh_attribute("POSITION"),
        blake3::hash(b"p"),
        36,
    );

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&Mesh3d>();

    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);

    ctx.remove_attr::<MeshAttr>(root);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert!(res.is_empty());
}

const POSITIONS: [[f32; 3]; 3] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
const NORMALS: [[f32; 3]; 3] = [[0.0, 0.0, 1.0]; 3];
const UVS: [[f32; 2]; 3] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
const INDICES: [u32; 3] = [0, 1, 2];

#[traced_test]
#[rstest]
fn test_mesh_blob_load(#[from(ctx_wds)] mut ctx: TestContext) {
    let positions = bytemuck::cast_slice::<[f32; 3], u8>(&POSITIONS).to_vec();
    let normals = bytemuck::cast_slice::<[f32; 3], u8>(&NORMALS).to_vec();
    let uvs = bytemuck::cast_slice::<[f32; 2], u8>(&UVS).to_vec();
    let indices = bytemuck::cast_slice::<u32, u8>(&INDICES).to_vec();

    let sizes = [positions.len(), normals.len(), uvs.len(), indices.len()];
    let pos_hash = ctx.upload_blob(positions);
    let norm_hash = ctx.upload_blob(normals);
    let uv_hash = ctx.upload_blob(uvs);
    let idx_hash = ctx.upload_blob(indices);

    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &MeshAttr {
            topology: Topology::TriangleList,
        },
    );
    ctx.set_bulk(
        root,
        &slots::mesh_attribute("POSITION"),
        pos_hash,
        sizes[0] as u64,
    );
    ctx.set_bulk(
        root,
        &slots::mesh_attribute("NORMAL"),
        norm_hash,
        sizes[1] as u64,
    );
    ctx.set_bulk(
        root,
        &slots::mesh_attribute("UV_0"),
        uv_hash,
        sizes[2] as u64,
    );
    ctx.set_bulk(root, slots::MESH_INDICES, idx_hash, sizes[3] as u64);

    let mut handle: Option<Handle<Mesh>> = None;
    ctx.tick_until(|world| {
        let mut q = world.query::<&Mesh3d>();
        let Some(m) = q.iter(world).next() else {
            return false;
        };
        if m.0 != Handle::<Mesh>::default() {
            handle = Some(m.0.clone());
            return true;
        }
        false
    });

    let handle = handle.expect("mesh handle");
    let assets = ctx.app.world().resource::<Assets<Mesh>>();
    let mesh = assets.get(&handle).expect("mesh asset");

    let pos = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("POSITION attribute");
    let VertexAttributeValues::Float32x3(pos) = pos else {
        panic!("POSITION wrong type");
    };
    assert_eq!(pos.as_slice(), &POSITIONS);

    let norm = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .expect("NORMAL attribute");
    let VertexAttributeValues::Float32x3(norm) = norm else {
        panic!("NORMAL wrong type");
    };
    assert_eq!(norm.as_slice(), &NORMALS);

    let uv = mesh
        .attribute(Mesh::ATTRIBUTE_UV_0)
        .expect("UV_0 attribute");
    let VertexAttributeValues::Float32x2(uv) = uv else {
        panic!("UV_0 wrong type");
    };
    assert_eq!(uv.as_slice(), &UVS);

    let Some(Indices::U32(idx)) = mesh.indices() else {
        panic!("indices missing or wrong type");
    };
    assert_eq!(idx.as_slice(), &INDICES);
}
