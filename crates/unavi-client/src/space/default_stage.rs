use std::collections::HashSet;

use bevy::{mesh::Indices, prelude::*};
use bevy_hsd::{
    hydration::topology::HydratedTopology,
    stage::{LayerData, OpinionData, StageData},
};
use blake3::Hash;
use bytemuck::cast_slice;
use bytes::Bytes;
use loro::{LoroMapValue, LoroValue};
use loro_surgeon::Reconcile;

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

pub fn default_stage() -> (Blobs, StageData) {
    let mut blobs = Blobs::default();

    let mut ground = new_cuboid(&mut blobs, Vec3::new(50.0, 1.0, 50.0));
    ground.make_mut().insert(
        "xform/pos".to_string(),
        LoroValue::List(
            vec![
                LoroValue::Double(0.0),
                LoroValue::Double(-1.0),
                LoroValue::Double(0.0),
            ]
            .into(),
        ),
    );
    ground.make_mut().insert(
        "rigid_body/kind".to_string(),
        LoroValue::String("static".into()),
    );

    let mut dyn_cube = new_cuboid(&mut blobs, Vec3::splat(0.5));
    dyn_cube.make_mut().insert(
        "xform/pos".to_string(),
        LoroValue::List(
            vec![
                LoroValue::Double(-2.0),
                LoroValue::Double(5.0),
                LoroValue::Double(-10.0),
            ]
            .into(),
        ),
    );
    dyn_cube.make_mut().insert(
        "rigid_body/kind".to_string(),
        LoroValue::String("dynamic".into()),
    );

    let stage = StageData {
        layers: vec![LayerData {
            enabled: true,
            opinions: vec![
                OpinionData {
                    node: 0,
                    attrs: ground,
                },
                OpinionData {
                    node: 1,
                    attrs: dyn_cube,
                },
            ],
        }],
    };

    (blobs, stage)
}

fn new_cuboid(blobs: &mut Blobs, dims: Vec3) -> LoroMapValue {
    let mut attrs = LoroMapValue::default();
    let cube = Cuboid::new(dims.x, dims.y, dims.z).mesh().build();

    attrs.make_mut().insert(
        "mesh/topology".to_string(),
        HydratedTopology(cube.primitive_topology())
            .to_loro_value()
            .expect("always exists"),
    );

    let Some(Indices::U32(indices)) = cube.indices() else {
        unreachable!()
    };
    attrs.make_mut().insert(
        "mesh/indices".to_string(),
        LoroValue::Binary(
            blobs
                .add_blob(cast_slice(indices).to_vec())
                .as_bytes()
                .to_vec()
                .into(),
        ),
    );

    let Some(points) = cube.attribute(Mesh::ATTRIBUTE_POSITION) else {
        unreachable!()
    };
    attrs.make_mut().insert(
        "mesh/points".to_string(),
        LoroValue::Binary(
            blobs
                .add_blob(points.get_bytes().to_vec())
                .as_bytes()
                .to_vec()
                .into(),
        ),
    );

    let Some(normals) = cube.attribute(Mesh::ATTRIBUTE_NORMAL) else {
        unreachable!()
    };
    attrs.make_mut().insert(
        "mesh/normals".to_string(),
        LoroValue::Binary(
            blobs
                .add_blob(normals.get_bytes().to_vec())
                .as_bytes()
                .to_vec()
                .into(),
        ),
    );

    attrs.make_mut().insert(
        "collider/shape".to_string(),
        LoroValue::String("cuboid".into()),
    );
    attrs.make_mut().insert(
        "collider/params".to_string(),
        LoroValue::List(
            vec![
                LoroValue::Double(f64::from(dims.x)),
                LoroValue::Double(f64::from(dims.y)),
                LoroValue::Double(f64::from(dims.z)),
            ]
            .into(),
        ),
    );

    attrs
}
