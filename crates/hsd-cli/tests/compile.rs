use std::{
    collections::HashMap,
    path::{
        Path,
        PathBuf,
    },
};

use hsd::{
    attributes::{
        material,
        name::NameAttr,
        slots,
    },
    id::BlobId,
    package::{
        self,
        Package,
    },
    state::{
        SceneState,
        entry::Entry,
    },
};
use hsd_cli::compile;

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

fn compile(input: &Path) -> anyhow::Result<Package> {
    compile::compile_file(input, &mut HashMap::new())
}

/// The package is bytes on disk before it is state, so the test goes through
/// the encoding rather than around it.
fn realize(package: &Package) -> SceneState {
    let bytes = package.encode().expect("encode");
    let package = Package::decode(&bytes).expect("decode");
    let (inline, bulk) = package::split(package);

    let mut state = SceneState::new();
    for (key, value) in inline {
        state.apply(&Entry::bytes(key, value, 1)).expect("apply");
    }
    for (key, value) in bulk {
        let size = value.len() as u64;
        let hash = BlobId(*blake3::hash(&value).as_bytes());
        state
            .apply(&Entry::blob(key, hash, size, 1))
            .expect("apply bulk");
    }
    state
}

fn prim_named(state: &SceneState, name: &str) -> hsd::id::PrimId {
    state
        .prims()
        .find(|prim| {
            state
                .attribute::<NameAttr>(*prim)
                .and_then(Result::ok)
                .is_some_and(|n| n.0 == name)
        })
        .unwrap_or_else(|| panic!("no prim named {name}"))
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
fn an_image_file_compiles_to_a_bulk_entry() {
    let state = realize(&compile(&write_source("image", SOURCE)).expect("compile"));

    let bulk = state
        .get(prim_named(&state, "tex"))
        .and_then(|prim| prim.bulk(slots::IMAGE_DATA))
        .expect("image bulk");

    assert_eq!(bulk.size, TEXTURE.len() as u64);
    assert_eq!(bulk.hash, BlobId(*blake3::hash(TEXTURE).as_bytes()));
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
