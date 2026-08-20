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
    ctx.set_slot(root, &slots::mesh_attribute("POSITION"), vec![0u8; 36]);

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&Mesh3d>();

    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);

    ctx.remove_attr::<MeshAttr>(root);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 0);
}

const POSITIONS: [[f32; 3]; 3] = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
const NORMALS: [[f32; 3]; 3] = [[0.0, 0.0, 1.0]; 3];
const UVS: [[f32; 2]; 3] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
const INDICES: [u32; 3] = [0, 1, 2];

#[traced_test]
#[rstest]
fn test_mesh_build(#[from(ctx_wds)] mut ctx: TestContext) {
    let positions = bytemuck::cast_slice::<[f32; 3], u8>(&POSITIONS).to_vec();
    let normals = bytemuck::cast_slice::<[f32; 3], u8>(&NORMALS).to_vec();
    let uvs = bytemuck::cast_slice::<[f32; 2], u8>(&UVS).to_vec();
    let indices = bytemuck::cast_slice::<u32, u8>(&INDICES).to_vec();

    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &MeshAttr {
            topology: Topology::TriangleList,
        },
    );
    ctx.set_slot(root, &slots::mesh_attribute("POSITION"), positions);
    ctx.set_slot(root, &slots::mesh_attribute("NORMAL"), normals);
    ctx.set_slot(root, &slots::mesh_attribute("UV_0"), uvs);
    ctx.set_slot(root, slots::MESH_INDICES, indices);

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

/// An index past the end of the vertex buffer is an out-of-bounds GPU read at
/// draw time, so the whole mesh is refused rather than uploaded.
#[traced_test]
#[rstest]
fn test_out_of_range_indices_are_rejected(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &MeshAttr {
            topology: Topology::TriangleList,
        },
    );
    ctx.set_slot(
        root,
        &slots::mesh_attribute("POSITION"),
        bytemuck::cast_slice(&POSITIONS).to_vec(),
    );
    ctx.set_slot(
        root,
        slots::MESH_INDICES,
        bytemuck::cast_slice(&[0u32, 1, 9]).to_vec(),
    );

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&Mesh3d>();
    let meshes = world.resource::<Assets<Mesh>>();
    assert!(
        query
            .query(world)
            .into_iter()
            .all(|handle| meshes.get(&handle.0).is_none()),
        "no mesh asset is built from out-of-range indices"
    );
}

/// Bevy's vertex-buffer assembly fails on attributes of differing lengths, so
/// a mismatched pair never reaches it.
#[traced_test]
#[rstest]
fn test_mismatched_attribute_lengths_are_rejected(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &MeshAttr {
            topology: Topology::TriangleList,
        },
    );
    ctx.set_slot(
        root,
        &slots::mesh_attribute("POSITION"),
        bytemuck::cast_slice(&POSITIONS).to_vec(),
    );
    ctx.set_slot(
        root,
        &slots::mesh_attribute("NORMAL"),
        bytemuck::cast_slice(&[[0.0f32, 1.0, 0.0]]).to_vec(),
    );

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&Mesh3d>();
    let meshes = world.resource::<Assets<Mesh>>();
    assert!(
        query
            .query(world)
            .into_iter()
            .all(|handle| meshes.get(&handle.0).is_none()),
        "no mesh asset is built from mismatched attribute lengths"
    );
}

/// The control for the two rejection tests: the same shape, with indices in
/// range, does build an asset.
#[traced_test]
#[rstest]
fn test_in_range_indices_build_a_mesh(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &MeshAttr {
            topology: Topology::TriangleList,
        },
    );
    ctx.set_slot(
        root,
        &slots::mesh_attribute("POSITION"),
        bytemuck::cast_slice(&POSITIONS).to_vec(),
    );
    ctx.set_slot(
        root,
        slots::MESH_INDICES,
        bytemuck::cast_slice(&[0u32, 1, 2]).to_vec(),
    );

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&Mesh3d>();
    let handles = query.query(world).into_iter().cloned().collect::<Vec<_>>();
    let meshes = world.resource::<Assets<Mesh>>();
    assert!(
        handles.iter().any(|handle| meshes.get(&handle.0).is_some()),
        "a valid mesh reaches the asset store"
    );
}
