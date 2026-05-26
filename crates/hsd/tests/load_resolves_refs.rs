use std::collections::BTreeMap;

use hsd::{
    HSD_CONTAINER_ID,
    attributes::{
        Attribute, Attributes, attributes_map, image::ImageAttr, material::MaterialAttr,
        name::NameAttr, relationships_map,
    },
    file::{HsdFile, HsdFilePrim},
};
use loro::{LoroDoc, TreeID};
use loro_surgeon::bytes::ByteArray;
fn doc_with_file(file: &HsdFile) -> LoroDoc {
    let doc = LoroDoc::new();
    file.load_into_doc(&doc).expect("load");
    doc
}

#[test]
fn material_texture_name_is_resolved_to_tree_id() {
    let file = HsdFile(vec![
        HsdFilePrim {
            attributes: Attributes {
                name: Some(NameAttr("tex".into())),
                image: Some(ImageAttr {
                    data: ByteArray::new([7u8; 32]),
                    srgb: Some(true),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        HsdFilePrim {
            attributes: Attributes {
                name: Some(NameAttr("mat".into())),
                material: Some(MaterialAttr {
                    base_color_texture: Some("tex".into()),
                    emissive_texture: Some("tex".into()),
                    normal_texture: Some("missing".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
    ]);

    let doc = doc_with_file(&file);
    let tree = doc.get_tree(&*HSD_CONTAINER_ID);

    let roots = tree.roots();
    assert_eq!(roots.len(), 2);
    let tex_id = roots[0];
    let mat_id = roots[1];

    let mat_meta = tree.get_meta(mat_id).expect("mat meta");
    let mat_attr = MaterialAttr::attr_hydrate(&attributes_map(&mat_meta).expect("attrs"))
        .expect("hydrate material");

    let base = mat_attr
        .base_color_texture
        .expect("base_color_texture missing");
    let parsed = TreeID::try_from(base.as_str()).expect("base_color_texture is a TreeID");
    assert_eq!(parsed, tex_id);

    let emissive = mat_attr.emissive_texture.expect("emissive_texture missing");
    assert_eq!(
        TreeID::try_from(emissive.as_str()).expect("emissive is a TreeID"),
        tex_id,
    );

    let normal = mat_attr.normal_texture.expect("normal_texture missing");
    assert_eq!(normal, "missing", "unknown refs pass through unchanged");
    assert!(TreeID::try_from(normal.as_str()).is_err());
}

#[test]
fn relationship_name_is_resolved_to_tree_id() {
    let file = HsdFile(vec![
        HsdFilePrim {
            attributes: Attributes {
                name: Some(NameAttr("target".into())),
                ..Default::default()
            },
            ..Default::default()
        },
        HsdFilePrim {
            attributes: Attributes {
                name: Some(NameAttr("source".into())),
                ..Default::default()
            },
            relationships: BTreeMap::from([("material".to_string(), "target".to_string())]),
            ..Default::default()
        },
    ]);

    let doc = doc_with_file(&file);
    let tree = doc.get_tree(&*HSD_CONTAINER_ID);
    let roots = tree.roots();
    let target_id = roots[0];
    let source_id = roots[1];

    let source_meta = tree.get_meta(source_id).expect("source meta");
    let rels = relationships_map(&source_meta).expect("relationships");
    let target_str = match rels.get("material").expect("material rel") {
        loro::ValueOrContainer::Value(loro::LoroValue::String(s)) => s.to_string(),
        other => panic!("unexpected value: {other:?}"),
    };

    let parsed = TreeID::try_from(target_str.as_str()).expect("rel is TreeID");
    assert_eq!(parsed, target_id);
}

#[test]
fn round_trip_through_ron() {
    let file = HsdFile(vec![HsdFilePrim {
        attributes: Attributes {
            name: Some(NameAttr("tex".into())),
            image: Some(ImageAttr {
                data: ByteArray::new([3u8; 32]),
                srgb: Some(true),
                ..Default::default()
            }),
            material: Some(MaterialAttr {
                base_color_texture: Some("tex".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }]);

    let ron = file.to_ron().expect("to_ron");
    let parsed = HsdFile::from_ron(&ron).expect("from_ron");
    let ron2 = parsed.to_ron().expect("to_ron 2");
    assert_eq!(ron, ron2);
}
