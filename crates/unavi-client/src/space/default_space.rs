use std::collections::HashSet;

use bevy::{mesh::Indices, prelude::*};
use blake3::Hash;
use bytemuck::cast_slice;
use bytes::Bytes;
use loro::{LoroList, LoroMap, LoroTree};

#[derive(Default)]
pub struct Blobs(pub HashSet<Bytes>);

impl Blobs {
    fn add_blob(&mut self, bytes: impl Into<Bytes>) -> Hash {
        let bytes = bytes.into();
        let hash = blake3::hash(&bytes);
        self.0.insert(bytes);
        hash
    }
}

/// Write default HSD scene into the provided map.
/// Returns blob data that must be uploaded.
pub fn default_space(hsd: &LoroMap) -> Blobs {
    let mut blobs = Blobs::default();

    let materials = hsd
        .get_or_create_container("materials", LoroList::new())
        .expect("materials list");

    // Ground material (index 0)
    let mat0 = materials.push_container(LoroMap::new()).expect("push mat");
    mat0.insert_container("base_color", {
        let l = LoroList::new();
        l.push(0.5).expect("push");
        l.push(0.5).expect("push");
        l.push(0.5).expect("push");
        l
    })
    .expect("base_color");
    mat0.insert("roughness", 0.9).expect("roughness");

    let meshes = hsd
        .get_or_create_container("meshes", LoroList::new())
        .expect("meshes list");

    // Ground mesh (index 0).
    let ground_dims = Vec3::new(50.0, 1.0, 50.0);
    let ground_mesh_idx = push_cuboid_mesh(&mut blobs, &meshes, ground_dims);

    // Dyn cube mesh (index 1).
    let cube_dims = Vec3::splat(0.5);
    let cube_mesh_idx = push_cuboid_mesh(&mut blobs, &meshes, cube_dims);

    let nodes = hsd
        .get_or_create_container("nodes", LoroTree::new())
        .expect("nodes tree");

    // Ground node.
    let ground_id = nodes.create(None).expect("create ground");
    let ground = nodes.get_meta(ground_id).expect("meta");
    ground.insert("mesh", ground_mesh_idx).expect("mesh");
    ground.insert("material", 0i64).expect("material");
    ground
        .insert_container("translation", {
            let l = LoroList::new();
            l.push(0.0).expect("push");
            l.push(-1.0).expect("push");
            l.push(0.0).expect("push");
            l
        })
        .expect("translation");
    let ground_collider = ground
        .get_or_create_container("collider", LoroMap::new())
        .expect("collider");
    ground_collider.insert("shape", "cuboid").expect("shape");
    ground_collider
        .insert_container("size", {
            let l = LoroList::new();
            l.push(f64::from(ground_dims.x)).expect("push");
            l.push(f64::from(ground_dims.y)).expect("push");
            l.push(f64::from(ground_dims.z)).expect("push");
            l
        })
        .expect("size");
    let ground_rb = ground
        .get_or_create_container("rigid_body", LoroMap::new())
        .expect("rigid_body");
    ground_rb.insert("kind", "static").expect("kind");

    // Dynamic cube node.
    let cube_id = nodes.create(None).expect("create cube");
    let cube = nodes.get_meta(cube_id).expect("meta");
    cube.insert("mesh", cube_mesh_idx).expect("mesh");
    cube.insert_container("translation", {
        let l = LoroList::new();
        l.push(-2.0).expect("push");
        l.push(5.0).expect("push");
        l.push(-10.0).expect("push");
        l
    })
    .expect("translation");
    let cube_collider = cube
        .get_or_create_container("collider", LoroMap::new())
        .expect("collider");
    cube_collider.insert("shape", "cuboid").expect("shape");
    cube_collider
        .insert_container("size", {
            let l = LoroList::new();
            l.push(f64::from(cube_dims.x)).expect("push");
            l.push(f64::from(cube_dims.y)).expect("push");
            l.push(f64::from(cube_dims.z)).expect("push");
            l
        })
        .expect("size");
    let cube_rb = cube
        .get_or_create_container("rigid_body", LoroMap::new())
        .expect("rigid_body");
    cube_rb.insert("kind", "dynamic").expect("kind");

    blobs
}

/// Push a cuboid mesh into the meshes list, returning the
/// index (as i64 for Loro).
fn push_cuboid_mesh(blobs: &mut Blobs, meshes: &LoroList, dims: Vec3) -> i64 {
    let cube = Cuboid::new(dims.x, dims.y, dims.z).mesh().build();

    let mesh_map = meshes.push_container(LoroMap::new()).expect("push mesh");
    mesh_map.insert("topology", 3i64).expect("topology");

    let attrs = mesh_map
        .get_or_create_container("attributes", LoroMap::new())
        .expect("attrs");

    // Indices.
    let Some(Indices::U32(indices)) = cube.indices() else {
        unreachable!()
    };
    let hash = blobs.add_blob(cast_slice(indices).to_vec());
    mesh_map
        .insert("indices", hash.as_bytes().to_vec())
        .expect("indices");

    // Position.
    let Some(points) = cube.attribute(Mesh::ATTRIBUTE_POSITION) else {
        unreachable!()
    };
    let hash = blobs.add_blob(points.get_bytes().to_vec());
    attrs
        .insert("POSITION", hash.as_bytes().to_vec())
        .expect("position");

    // Normals.
    let Some(normals) = cube.attribute(Mesh::ATTRIBUTE_NORMAL) else {
        unreachable!()
    };
    let hash = blobs.add_blob(normals.get_bytes().to_vec());
    attrs
        .insert("NORMAL", hash.as_bytes().to_vec())
        .expect("normal");

    // Return index (0-based).
    #[expect(clippy::cast_possible_wrap)]
    let idx = (meshes.len() - 1) as i64;
    idx
}
