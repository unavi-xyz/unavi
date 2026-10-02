//! Primitive mesh builders, composed into a consumer at build time.
//!
//! Each resource holds its own tessellation parameters and builds fresh
//! triangle data on every `mesh()` call, written as a new prim in the
//! document named by `set-doc`, or the calling script's own document by
//! default.

use std::cell::RefCell;

use exports::unavi::shapes::api::Guest;
use wired::{
    core::error::Error,
    scene::{
        document::{
            Document,
            Layer,
            script_document,
        },
        properties::{
            Property,
            Topology,
            VertexAttribute,
            VertexStream,
        },
    },
};

mod capsule;
mod cone;
mod cuboid;
mod cylinder;
mod sphere;
mod torus;

wired_guest::generate!();

struct World;

impl Guest for World {
    type Capsule = capsule::Capsule;
    type Cone = cone::Cone;
    type Cuboid = cuboid::Cuboid;
    type Cylinder = cylinder::Cylinder;
    type Sphere = sphere::Sphere;
    type Torus = torus::Torus;
}

/// Positions, normals and UVs, one entry per vertex, plus triangle indices.
struct RawMesh {
    positions: Vec<[f32; 3]>,
    normals:   Vec<[f32; 3]>,
    uvs:       Vec<[f32; 2]>,
    indices:   Vec<u32>,
}

/// Builds `raw` into `doc`, or the calling script's own document when `doc`
/// holds none (the default before `set-doc`).
fn mesh_into(doc: &RefCell<Option<Document>>, raw: &RawMesh) -> Result<(u64, u64), Error> {
    match doc.borrow().as_ref() {
        Some(doc) => convert_raw_mesh(doc, raw),
        None => convert_raw_mesh(&script_document()?, raw),
    }
}

/// Writes `raw` as a new, parentless prim in `doc`.
fn convert_raw_mesh(doc: &Document, raw: &RawMesh) -> Result<(u64, u64), Error> {
    let prim = doc.create_prim(Layer::Local, None)?;

    doc.local()
        .set(prim, Property::MeshTopology(Topology::TriangleList))
        .set(
            prim,
            Property::MeshVertices(VertexStream {
                attribute: VertexAttribute::Position,
                values:    raw.positions.as_flattened().to_vec(),
            }),
        )
        .set(
            prim,
            Property::MeshVertices(VertexStream {
                attribute: VertexAttribute::Normal,
                values:    raw.normals.as_flattened().to_vec(),
            }),
        )
        .set(
            prim,
            Property::MeshVertices(VertexStream {
                attribute: VertexAttribute::Uv0,
                values:    raw.uvs.as_flattened().to_vec(),
            }),
        )
        .set(prim, Property::MeshIndices(raw.indices.clone()))
        .flush()?;

    Ok(prim)
}

export!(World);
