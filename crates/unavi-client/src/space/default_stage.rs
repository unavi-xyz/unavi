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
pub struct Blobs(pub Vec<Bytes>);

impl Blobs {
    fn add_blob(&mut self, bytes: impl Into<Bytes>) -> Hash {
        let bytes = bytes.into();
        let hash = blake3::hash(&bytes);
        self.0.push(bytes);
        hash
    }
}

pub fn default_stage() -> (Blobs, StageData) {
    let mut attrs = LoroMapValue::default();
    let mut blobs = Blobs::default();

    // Ground.
    let x_length = 20.0;
    let y_length = 1.0;
    let z_length = 20.0;

    attrs.make_mut().insert(
        "xform/pos".to_string(),
        LoroValue::List(
            vec![
                LoroValue::Double(0.0),
                LoroValue::Double(f64::from(y_length) / -2.0 - 1.0),
                LoroValue::Double(0.0),
            ]
            .into(),
        ),
    );

    let cube = Cuboid::new(x_length, y_length, z_length).mesh().build();

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
                LoroValue::Double(f64::from(x_length)),
                LoroValue::Double(f64::from(y_length)),
                LoroValue::Double(f64::from(z_length)),
            ]
            .into(),
        ),
    );

    let stage = StageData {
        layers: vec![LayerData {
            enabled: true,
            opinions: vec![OpinionData { node: 0, attrs }],
        }],
    };

    (blobs, stage)
}
