use std::collections::BTreeMap;

use bevy::{
    mesh::{Indices, VertexAttributeValues},
    prelude::*,
};
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, mesh::MeshAttr},
};
use lorosurgeon::ByteArray;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_mesh_lifecycle(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    // Add attribute.
    let attr = MeshAttr {
        attributes: BTreeMap::from([("POSITION".to_string(), ByteArray::<32>::new([1; 32]))]),
        indices: None,
        topology: 3,
    };
    attr.attr_reconcile(meta.clone()).expect("reconcile");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&Mesh3d>();

    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);

    // Remove attribute.
    meta.delete(MeshAttr::KEY).expect("delete");

    ctx.doc.commit();
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
    let pos_hash = ctx.upload_blob(bytemuck::cast_slice::<[f32; 3], u8>(&POSITIONS).to_vec());
    let norm_hash = ctx.upload_blob(bytemuck::cast_slice::<[f32; 3], u8>(&NORMALS).to_vec());
    let uv_hash = ctx.upload_blob(bytemuck::cast_slice::<[f32; 2], u8>(&UVS).to_vec());
    let idx_hash = ctx.upload_blob(bytemuck::cast_slice::<u32, u8>(&INDICES).to_vec());

    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");
    let attr = MeshAttr {
        attributes: BTreeMap::from([
            (
                "POSITION".to_string(),
                ByteArray::<32>::new(*pos_hash.as_bytes()),
            ),
            (
                "NORMAL".to_string(),
                ByteArray::<32>::new(*norm_hash.as_bytes()),
            ),
            (
                "UV_0".to_string(),
                ByteArray::<32>::new(*uv_hash.as_bytes()),
            ),
        ]),
        indices: Some(ByteArray::<32>::new(*idx_hash.as_bytes())),
        topology: 3,
    };
    attr.attr_reconcile(meta).expect("reconcile");

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
