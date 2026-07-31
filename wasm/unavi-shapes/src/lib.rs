use crate::{
    exports::unavi::shapes::api::Guest,
    wired::scene::types::{
        Document,
        Prim,
    },
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
    normals:   Vec<[f32; 3]>,
    uvs:       Vec<[f32; 2]>,
    indices:   Vec<u32>,
}

fn convert_raw_mesh(doc: Option<&Document>, raw: RawMesh) -> Prim {
    let prim = match doc {
        Some(doc) => doc.create_prim(),
        None => wired::scene::api::self_document()
            .expect("self_document")
            .create_prim(),
    }
    .expect("create_prim");
    prim.set_mesh_stream("POSITION", Some(raw.positions.as_flattened()))
        .ok();
    prim.set_mesh_stream("NORMAL", Some(raw.normals.as_flattened()))
        .ok();
    prim.set_mesh_stream("UV_0", Some(raw.uvs.as_flattened()))
        .ok();
    prim.set_mesh_indices_u32(Some(&raw.indices)).ok();
    prim
}

export!(World);
