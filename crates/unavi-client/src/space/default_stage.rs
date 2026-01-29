use bevy::{mesh::Indices, prelude::*};
use bevy_hsd::stage::{LayerData, OpinionData, StageData};
use bytemuck::cast_slice;
use loro::{LoroMapValue, LoroValue};

pub fn default_stage() -> StageData {
    let mut attrs = LoroMapValue::default();

    // Ground.
    let x_length = 10.0;
    let y_length = 0.5;
    let z_length = 10.0;

    attrs.make_mut().insert(
        "xform/pos".to_string(),
        LoroValue::List(
            vec![
                LoroValue::Double(0.0),
                LoroValue::Double(f64::from(y_length) / -2.0),
                LoroValue::Double(0.0),
            ]
            .into(),
        ),
    );

    attrs.make_mut().insert(
        "mesh/topology".to_string(),
        LoroValue::I64(
            3, // TriangleList
        ),
    );

    let cube = Cuboid::new(x_length, y_length, z_length).mesh().build();

    let Some(Indices::U32(indices)) = cube.indices() else {
        unreachable!()
    };
    attrs.make_mut().insert(
        "mesh/indices".to_string(),
        LoroValue::Binary(cast_slice(indices).to_vec().into()),
    );

    let Some(points) = cube.attribute(Mesh::ATTRIBUTE_POSITION) else {
        unreachable!()
    };
    attrs.make_mut().insert(
        "mesh/points".to_string(),
        LoroValue::Binary(points.get_bytes().to_vec().into()),
    );

    let Some(normals) = cube.attribute(Mesh::ATTRIBUTE_NORMAL) else {
        unreachable!()
    };
    attrs.make_mut().insert(
        "mesh/normals".to_string(),
        LoroValue::Binary(normals.get_bytes().to_vec().into()),
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

    StageData {
        layers: vec![LayerData {
            enabled: true,
            opinions: vec![OpinionData { node: 0, attrs }],
        }],
    }
}
