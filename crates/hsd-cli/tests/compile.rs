mod common;

use std::{
    collections::HashMap,
    path::{
        Path,
        PathBuf,
    },
};

use common::{
    compile,
    prim_named,
    realize,
};
use hsd::{
    attributes::{
        material,
        reference::ReferenceAttr,
        slots,
    },
    id::DocId,
    package::Package,
    state::{
        HsdState,
        entry::Entry,
    },
};

const SOURCE: &str = r#"[
    (
        attributes: (name: "root"),
        children: [
            (
                attributes: (
                    name: "cube",
                    material: (base_color: [1.0, 0.0, 0.0, 1.0], base_color_texture: "tex"),
                ),
            ),
            (attributes: (name: "tex", image: (data: "tex.png"))),
        ],
    ),
]"#;

const TEXTURE: &[u8] = b"not really a png";

fn write_source(case: &str, hsda: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(case);
    std::fs::create_dir_all(&dir).expect("create case dir");
    std::fs::write(dir.join("tex.png"), TEXTURE).expect("write texture");
    let input = dir.join("asset.hsda");
    std::fs::write(&input, hsda).expect("write source");
    input
}

#[test]
fn a_compiled_source_tree_realizes_as_authored() {
    let state = realize(&compile(&write_source("tree", SOURCE)).expect("compile"));

    let root = prim_named(&state, "root");
    let cube = prim_named(&state, "cube");
    let tex = prim_named(&state, "tex");

    assert!(state.is_realized(root));
    assert_eq!(state.parent(root), None);
    assert_eq!(state.parent(cube), Some(root));
    assert_eq!(state.children(root), vec![cube, tex]);
}

#[test]
fn a_texture_field_compiles_to_a_relationship() {
    let state = realize(&compile(&write_source("texture", SOURCE)).expect("compile"));

    assert_eq!(
        state.relationship(prim_named(&state, "cube"), material::BASE_COLOR_TEXTURE),
        Some(prim_named(&state, "tex"))
    );
}

#[test]
fn an_image_file_compiles_to_a_slot_entry() {
    let state = realize(&compile(&write_source("image", SOURCE)).expect("compile"));

    let bytes = state
        .get(prim_named(&state, "tex"))
        .and_then(|prim| prim.slot(slots::IMAGE_DATA))
        .expect("image slot");

    assert_eq!(bytes, TEXTURE);
}

/// Ids come from the source path and tree position, so an unchanged input
/// compiles to the same package on every machine.
#[test]
fn compilation_is_reproducible() {
    let input = write_source("reproducible", SOURCE);
    assert_eq!(
        compile(&input).expect("compile").encode().expect("encode"),
        compile(&input).expect("compile").encode().expect("encode")
    );
}

#[test]
fn a_dangling_reference_fails_the_build() {
    let source = SOURCE.replace("\"tex\")", "\"missing\")");
    let err = compile(&write_source("dangling", &source)).expect_err("should fail");
    assert!(err.to_string().contains("missing"), "{err}");
}

#[test]
fn a_duplicate_name_fails_the_build() {
    let source = SOURCE.replace("name: \"cube\"", "name: \"tex\"");
    let err = compile(&write_source("duplicate", &source)).expect_err("should fail");
    assert!(err.to_string().contains("duplicate"), "{err}");
}

const TARGET: &str = r#"[(attributes: (name: "chair"))]"#;

/// Two prims naming one file, so the package has to carry the target once and
/// both references have to name that one copy.
const REFERENCING: &str = r#"[
    (
        attributes: (name: "room"),
        children: [
            (attributes: (name: "left", reference: "chair/asset.hsda")),
            (attributes: (name: "right", reference: "chair/asset.hsda")),
        ],
    ),
]"#;

fn write_referencing(case: &str) -> PathBuf {
    let input = write_source(case, REFERENCING);
    let target_dir = input.parent().expect("case dir has a parent").join("chair");
    std::fs::create_dir_all(&target_dir).expect("create target dir");
    std::fs::write(target_dir.join("asset.hsda"), TARGET).expect("write target");
    input
}

#[test]
fn a_referenced_file_is_carried_beside_the_root_rather_than_inside_it() {
    let package = compile(&write_referencing("reference")).expect("compile");

    assert_eq!(
        package.documents.len(),
        1,
        "two prims naming one file share one copy of it; that is the dedup a \
         reference buys over an embedded package"
    );

    let state = realize(&package);
    let left = state
        .attribute::<ReferenceAttr>(prim_named(&state, "left"))
        .expect("left references something")
        .expect("decodes");
    let right = state
        .attribute::<ReferenceAttr>(prim_named(&state, "right"))
        .expect("right references something")
        .expect("decodes");

    assert_eq!(left, right, "both name the same placeholder");
    assert_eq!(
        left.0, package.documents[0].0,
        "and the placeholder is the one the package carries"
    );
}

#[test]
fn a_reference_costs_an_id_rather_than_the_target() {
    let package = compile(&write_referencing("reference-size")).expect("compile");

    let refs: Vec<_> = package
        .entries
        .iter()
        .filter(|(key, _)| key.ends_with("/ref/"))
        .collect();
    assert_eq!(refs.len(), 2, "one reference entry per referencing prim");
    for (_, value) in refs {
        assert_eq!(
            value.len(),
            33,
            "an id and its property tag, against the whole target an embedded \
             package used to carry"
        );
    }

    assert!(
        !package.documents[0].1.is_empty(),
        "and the target is carried, once, beside the root"
    );
}

/// Placeholders are meaningless outside the package that carries them, so a
/// reference the map cannot answer must fail rather than be written through to
/// name a namespace nobody can serve.
#[test]
fn rewriting_refuses_a_reference_the_package_does_not_carry() {
    let package = compile(&write_referencing("reference-dangling")).expect("compile");
    let mut entries = package.entries;

    assert!(
        Package::rewrite_refs(&mut entries, &HashMap::new()).is_err(),
        "an unmapped placeholder is an error, not a value written through"
    );
}

#[test]
fn rewriting_replaces_every_reference_with_its_minted_id() {
    let package = compile(&write_referencing("reference-rewrite")).expect("compile");
    let placeholder = package.documents[0].0;
    let minted = DocId([9; 32]);

    let mut entries = package.entries;
    Package::rewrite_refs(&mut entries, &HashMap::from([(placeholder, minted)])).expect("rewrite");

    let mut state = HsdState::new();
    for (key, value) in entries {
        state.apply(&Entry::new(key, value, 1)).expect("apply");
    }

    for name in ["left", "right"] {
        assert_eq!(
            state
                .attribute::<ReferenceAttr>(prim_named(&state, name))
                .expect("references something")
                .expect("decodes")
                .0,
            minted
        );
    }
}
