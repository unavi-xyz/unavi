use std::collections::HashSet;

use anyhow::Result;
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
pub fn default_space(hsd: &LoroMap) -> Result<Blobs> {
    let mut blobs = Blobs::default();

    let materials = hsd.get_or_create_container("materials", LoroMap::new())?;

    // Ground material
    let mat0 = materials.get_or_create_container("0", LoroMap::new())?;
    mat0.insert_container("base_color", {
        let l = LoroList::new();
        l.push(1.0)?;
        l.push(1.0)?;
        l.push(1.0)?;
        l
    })?;
    mat0.insert("roughness", 0.9)?;

    let meshes = hsd.get_or_create_container("meshes", LoroMap::new())?;

    // Ground mesh
    let ground_dims = Vec3::new(50.0, 1.0, 50.0);
    insert_cuboid_mesh(&mut blobs, &meshes, "0", ground_dims)?;

    // Dyn cube mesh
    let cube_dims = Vec3::splat(0.5);
    insert_cuboid_mesh(&mut blobs, &meshes, "1", cube_dims)?;

    let nodes = hsd.get_or_create_container("nodes", LoroTree::new())?;

    // Ground node
    let ground_id = nodes.create(None)?;
    let ground = nodes.get_meta(ground_id)?;
    ground.insert("mesh", "0")?;
    ground.insert("material", "0")?;
    ground.insert_container("translation", {
        let l = LoroList::new();
        l.push(0.0)?;
        l.push(-1.0)?;
        l.push(0.0)?;
        l
    })?;
    let ground_collider = ground.get_or_create_container("collider", LoroMap::new())?;
    ground_collider.insert("tag", "Cuboid")?;
    let ground_cuboid = ground_collider.get_or_create_container("Cuboid", LoroMap::new())?;
    ground_cuboid.insert("x", f64::from(ground_dims.x))?;
    ground_cuboid.insert("y", f64::from(ground_dims.y))?;
    ground_cuboid.insert("z", f64::from(ground_dims.z))?;
    let ground_rb = ground.get_or_create_container("rigid_body", LoroMap::new())?;
    ground_rb.insert("kind", "static")?;

    // Dynamic cube node
    let cube_id = nodes.create(None)?;
    let cube = nodes.get_meta(cube_id)?;
    cube.insert("mesh", "1")?;
    cube.insert_container("translation", {
        let l = LoroList::new();
        l.push(-2.0)?;
        l.push(5.0)?;
        l.push(-10.0)?;
        l
    })?;
    let cube_collider = cube.get_or_create_container("collider", LoroMap::new())?;
    cube_collider.insert("tag", "Cuboid")?;
    let cube_cuboid = cube_collider.get_or_create_container("Cuboid", LoroMap::new())?;
    cube_cuboid.insert("x", f64::from(cube_dims.x))?;
    cube_cuboid.insert("y", f64::from(cube_dims.y))?;
    cube_cuboid.insert("z", f64::from(cube_dims.z))?;
    let cube_rb = cube.get_or_create_container("rigid_body", LoroMap::new())?;
    cube_rb.insert("kind", "dynamic")?;

    Ok(blobs)
}

fn insert_cuboid_mesh(blobs: &mut Blobs, meshes: &LoroMap, key: &str, dims: Vec3) -> Result<()> {
    let cube = Cuboid::new(dims.x, dims.y, dims.z).mesh().build();

    let mesh_map = meshes.get_or_create_container(key, LoroMap::new())?;
    mesh_map.insert("topology", 3i64)?;

    let attrs = mesh_map.get_or_create_container("attributes", LoroMap::new())?;

    // Indices.
    let Some(Indices::U32(indices)) = cube.indices() else {
        unreachable!()
    };
    let hash = blobs.add_blob(cast_slice(indices).to_vec());
    mesh_map.insert("indices", hash.as_bytes().to_vec())?;

    // Position.
    let Some(points) = cube.attribute(Mesh::ATTRIBUTE_POSITION) else {
        unreachable!()
    };
    let hash = blobs.add_blob(points.get_bytes().to_vec());
    attrs.insert("POSITION", hash.as_bytes().to_vec())?;

    // Normals.
    let Some(normals) = cube.attribute(Mesh::ATTRIBUTE_NORMAL) else {
        unreachable!()
    };
    let hash = blobs.add_blob(normals.get_bytes().to_vec());
    attrs.insert("NORMAL", hash.as_bytes().to_vec())?;

    Ok(())
}
