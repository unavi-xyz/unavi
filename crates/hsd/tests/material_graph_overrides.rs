use std::collections::BTreeMap;

use hsd::attributes::{
    Attribute,
    material_graph::{
        ShaderGraph,
        overrides::{
            GraphOverridesAttr,
            OverridesError,
            validate_overrides,
        },
        value::{
            GraphValue,
            ValueKind,
        },
    },
};

#[test]
fn overrides_must_match_declared_public_input_kind() {
    let graph = ShaderGraph {
        public_inputs: vec![GraphValue::Float(1.0)],
        ..Default::default()
    };

    let ok = GraphOverridesAttr {
        overrides: BTreeMap::from([(0, GraphValue::Float(2.0))]),
    };
    assert_eq!(validate_overrides(&graph, &ok), Ok(()));

    let wrong_kind = GraphOverridesAttr {
        overrides: BTreeMap::from([(0, GraphValue::Vec3([0.0; 3]))]),
    };
    assert_eq!(
        validate_overrides(&graph, &wrong_kind),
        Err(OverridesError::TypeMismatch {
            index:    0,
            expected: ValueKind::Float,
            found:    ValueKind::Vec3,
        })
    );

    let unknown = GraphOverridesAttr {
        overrides: BTreeMap::from([(1, GraphValue::Float(2.0))]),
    };
    assert_eq!(
        validate_overrides(&graph, &unknown),
        Err(OverridesError::UnknownInput(1))
    );
}

#[test]
fn overrides_attribute_round_trips() {
    let attr = GraphOverridesAttr {
        overrides: BTreeMap::from([(0, GraphValue::Color([1.0, 0.0, 0.0, 1.0]))]),
    };
    let bytes = attr.encode().expect("encode");
    assert_eq!(GraphOverridesAttr::decode(&bytes).expect("decode"), attr);
}

/// `f32`'s own formatting renders a non-finite `NaN`/`inf`, which a WGSL
/// literal cannot carry — an override holding one must be refused.
#[test]
fn a_non_finite_override_is_rejected() {
    let graph = ShaderGraph {
        public_inputs: vec![GraphValue::Float(0.0)],
        ..Default::default()
    };
    let overrides = GraphOverridesAttr {
        overrides: BTreeMap::from([(0, GraphValue::Float(f32::NEG_INFINITY))]),
    };
    assert_eq!(
        validate_overrides(&graph, &overrides),
        Err(OverridesError::NonFinite(0))
    );
}
