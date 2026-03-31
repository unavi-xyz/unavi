use crate::{
    exports::unavi::shapes::api::Guest,
    wired::scene::types::{Indices, Mesh},
};

mod capsule;
mod cone;
mod cuboid;
mod cylinder;
mod sphere;
mod torus;

wired_prelude::generate!();

struct World;

impl Guest for World {
    type Capsule = capsule::CapsuleWrapped;
    type Cone = cone::ConeWrapped;
    type Cuboid = cuboid::CuboidWrapped;
    type Cylinder = cylinder::CylinderWrapped;
    type Sphere = sphere::SphereWrapped;
    type Torus = torus::TorusWrapped;
}

struct RawMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

fn convert_raw_mesh(raw: RawMesh) -> Mesh {
    let doc = wired::scene::context::self_document();
    let out = doc.create_mesh();
    out.set_positions(Some(raw.positions.as_flattened()));
    out.set_normals(Some(raw.normals.as_flattened()));
    out.set_uv0(Some(raw.uvs.as_flattened()));
    out.set_indices(Some(&Indices::Full(raw.indices)));
    out
}

export!(World);
