use std::collections::HashSet;

use anyhow::Result;
use bevy::{mesh::Indices, prelude::*};
use blake3::Hash;
use bytemuck::cast_slice;
use bytes::Bytes;
use loro::{LoroList, LoroMap, LoroTree};

#[derive(Default)]
pub struct BlobSet(pub HashSet<Bytes>);

impl BlobSet {
    fn add_blob(&mut self, bytes: impl Into<Bytes>) -> Hash {
        let bytes = bytes.into();
        let hash = blake3::hash(&bytes);
        self.0.insert(bytes);
        hash
    }
}

const GROUND_SIZE: f32 = 40.0;

// TODO load texture from bevy asset, dont include bytes
const DEV_WHITE_RAW: &[u8] = include_bytes!("../../assets/image/dev-white.png");

/// Write default HSD scene into the provided map.
/// Returns blob data that must be uploaded.
pub fn default_space(hsd: &LoroMap) -> Result<BlobSet> {
    let mut blobs = BlobSet::default();

    let key_ground = "ground";
    let key_dyncube = "dyncube";
    let key_ground_tex = "ground_tex";

    let images = hsd.get_or_create_container("images", LoroMap::new())?;

    // Ground texture image
    let ground_texture_hash = blobs.add_blob(DEV_WHITE_RAW.to_vec());
    let img0 = images.get_or_create_container(key_ground_tex, LoroMap::new())?;
    img0.insert("address_mode_u", 0i64)?;
    img0.insert("address_mode_v", 0i64)?;
    img0.insert("address_mode_w", 0i64)?;
    img0.insert("data", ground_texture_hash.as_bytes().to_vec())?;
    img0.insert("mag_filter", 1i64)?;
    img0.insert("min_filter", 1i64)?;
    img0.insert("srgb", true)?;

    let materials = hsd.get_or_create_container("materials", LoroMap::new())?;

    // Ground material
    let mat0 = materials.get_or_create_container(key_ground, LoroMap::new())?;
    mat0.insert("name", "Ground Material")?;
    mat0.insert_container("base_color", {
        let l = LoroList::new();
        l.push(0.9)?;
        l.push(0.8)?;
        l.push(0.95)?;
        l
    })?;
    mat0.insert("roughness", 0.9)?;
    mat0.insert("base_color_texture", key_ground_tex)?;

    let meshes = hsd.get_or_create_container("meshes", LoroMap::new())?;

    // Ground mesh
    let ground_dims = Vec3::new(GROUND_SIZE, 1.0, GROUND_SIZE);
    insert_cuboid_mesh(&mut blobs, &meshes, key_ground, ground_dims)?;

    // Dyn cube mesh
    let cube_dims = Vec3::splat(0.5);
    insert_cuboid_mesh(&mut blobs, &meshes, key_dyncube, cube_dims)?;

    let nodes = hsd.get_or_create_container("nodes", LoroTree::new())?;

    // Ground node
    let ground_id = nodes.create(None)?;
    let ground = nodes.get_meta(ground_id)?;
    ground.insert("name", "Ground Node")?;
    ground.insert("mesh", key_ground)?;
    ground.insert("material", key_ground)?;
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
    ground_rb.insert("kind", "fixed")?;

    // Dynamic cube node
    let cube_id = nodes.create(None)?;
    let cube = nodes.get_meta(cube_id)?;
    cube.insert("name", "Dyn Cube Node")?;
    cube.insert("mesh", key_dyncube)?;
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

fn insert_cuboid_mesh(blobs: &mut BlobSet, meshes: &LoroMap, key: &str, dims: Vec3) -> Result<()> {
    let cube = Cuboid::new(dims.x, dims.y, dims.z).mesh().build();

    let mesh_map = meshes.get_or_create_container(key, LoroMap::new())?;
    mesh_map.insert("topology", 3i64)?;

    let attrs = mesh_map.get_or_create_container("attributes", LoroMap::new())?;

    let Some(Indices::U32(indices)) = cube.indices() else {
        unreachable!()
    };
    let hash = blobs.add_blob(cast_slice(indices).to_vec());
    mesh_map.insert("indices", hash.as_bytes().to_vec())?;

    let Some(points) = cube.attribute(Mesh::ATTRIBUTE_POSITION) else {
        unreachable!()
    };
    let hash = blobs.add_blob(points.get_bytes().to_vec());
    attrs.insert("POSITION", hash.as_bytes().to_vec())?;

    let Some(normals) = cube.attribute(Mesh::ATTRIBUTE_NORMAL) else {
        unreachable!()
    };
    let hash = blobs.add_blob(normals.get_bytes().to_vec());
    attrs.insert("NORMAL", hash.as_bytes().to_vec())?;

    let Some(uv0) = cube.attribute(Mesh::ATTRIBUTE_UV_0) else {
        unreachable!()
    };
    let hash = blobs.add_blob(uv0.get_bytes().to_vec());
    attrs.insert("UV_0", hash.as_bytes().to_vec())?;

    Ok(())
}
