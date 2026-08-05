use std::{
    collections::HashMap,
    path::{
        Path,
        PathBuf,
    },
};

use hsd::{
    attributes::material_graph::{
        GraphOverridesAttr,
        GraphValue,
        ShaderGraph,
    },
    id::BlobId,
    key,
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

/// The checked-in example under `tests/fixtures/glow`: two prims sharing one
/// compiled shader graph, one taking its defaults, one overriding the rim
/// tint.
fn glow_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/glow/asset.hsda")
}

fn compile(input: &Path) -> anyhow::Result<Package> {
    compile::compile_file(input, &mut HashMap::new())
}

fn realize(package: &Package) -> SceneState {
    let (inline, bulk) = package::split(package.clone());

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
                .attribute::<hsd::attributes::name::NameAttr>(*prim)
                .and_then(Result::ok)
                .is_some_and(|n| n.0 == name)
        })
        .unwrap_or_else(|| panic!("no prim named {name}"))
}

/// Looks a bulk slot's raw bytes up straight from the package, since a
/// realized [`SceneState`] tracks only the blob hash/size a bulk entry
/// resolved to, not the bytes themselves.
fn bulk_bytes<'a>(package: &'a Package, prim: hsd::id::PrimId, slot: &str) -> &'a [u8] {
    let key = key::bulk(prim, slot);
    package
        .entries
        .iter()
        .find(|(k, _)| *k == key)
        .map_or_else(|| panic!("no bulk entry at {key}"), |(_, v)| v.as_slice())
}

#[test]
fn a_shader_graph_compiles_to_a_bulk_entry() {
    let package = compile(&glow_fixture()).expect("compile");
    let state = realize(&package);
    let prim = prim_named(&state, "default_glow");

    let bytes = bulk_bytes(&package, prim, hsd::attributes::slots::MATERIAL_GRAPH_DATA);
    let graph = ShaderGraph::decode(bytes).expect("decode graph");
    assert_eq!(
        graph.public_inputs,
        vec![GraphValue::Color([0.1, 0.6, 1.0, 1.0])]
    );
    assert_eq!(graph.surface.nodes.len(), 2);
    assert!(graph.displacement.is_none());
}

/// A prim that takes the graph's defaults has no reason to carry the
/// overrides attribute at all.
#[test]
fn a_prim_without_overrides_has_no_overrides_attribute() {
    let state = realize(&compile(&glow_fixture()).expect("compile"));
    let prim = prim_named(&state, "default_glow");
    assert!(state.attribute::<GraphOverridesAttr>(prim).is_none());
}

#[test]
fn overrides_compile_to_the_overrides_attribute() {
    let state = realize(&compile(&glow_fixture()).expect("compile"));
    let prim = prim_named(&state, "red_glow");

    let overrides = state
        .attribute::<GraphOverridesAttr>(prim)
        .expect("overrides present")
        .expect("decode overrides");
    assert_eq!(
        overrides.overrides.get(&0),
        Some(&GraphValue::Color([1.0, 0.0, 0.0, 1.0]))
    );
}

/// The dedup story this format depends on: two prims authored against the
/// same `.shader` file, one with overrides and one without, still compile
/// to byte-identical `material:graph_data` entries — overrides live in the
/// separate, small attribute, never in the bulk graph bytes.
#[test]
fn two_prims_sharing_a_graph_get_byte_identical_bulk_entries() {
    let package = compile(&glow_fixture()).expect("compile");
    let state = realize(&package);

    let a = bulk_bytes(
        &package,
        prim_named(&state, "default_glow"),
        hsd::attributes::slots::MATERIAL_GRAPH_DATA,
    );
    let b = bulk_bytes(
        &package,
        prim_named(&state, "red_glow"),
        hsd::attributes::slots::MATERIAL_GRAPH_DATA,
    );

    assert_eq!(
        a, b,
        "identical .shader source must compile byte-identically"
    );
}

#[test]
fn compilation_is_reproducible() {
    assert_eq!(
        compile(&glow_fixture())
            .expect("compile")
            .encode()
            .expect("encode"),
        compile(&glow_fixture())
            .expect("compile")
            .encode()
            .expect("encode")
    );
}

fn write_source(case: &str, shader: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(case);
    std::fs::create_dir_all(&dir).expect("create case dir");
    std::fs::write(dir.join("glow.shader"), shader).expect("write shader");
    let hsda = r#"[(attributes: (name: "p", material_graph: (path: "./glow.shader")))]"#;
    let input = dir.join("asset.hsda");
    std::fs::write(&input, hsda).expect("write source");
    input
}

/// A node referencing a later index cannot be constructed by the format
/// itself, but a hand-written `.shader` file can still spell it out — the
/// build must reject it rather than silently compiling an invalid graph.
#[test]
fn a_forward_reference_fails_the_build() {
    let shader = r"(
        surface: (
            nodes: [
                (kind: Add(a: Node(1), b: Const(Float(1.0)))),
                (kind: Time),
            ],
            output: Unlit((color: Const(Color((1.0, 1.0, 1.0, 1.0))))),
        ),
    )";
    let err = compile(&write_source("forward_ref", shader)).expect_err("should fail");
    assert!(err.to_string().contains("validating shader graph"), "{err}");
}

/// A node-count cap violation is a build failure, not a silently-truncated
/// graph.
#[test]
fn exceeding_the_node_cap_fails_the_build() {
    let nodes = "(kind: Time),".repeat(hsd::attributes::material_graph::MAX_NODES + 1);
    let shader = format!(
        "(surface: (nodes: [{nodes}], output: Unlit((color: Const(Color((1.0, 1.0, 1.0, 1.0)))))))"
    );
    let err = compile(&write_source("too_many_nodes", &shader)).expect_err("should fail");
    assert!(format!("{err:#}").contains("exceeding the cap"), "{err:#}");
}

/// An override whose kind does not match the public input it targets is
/// caught at compile time, not silently miscompiled.
#[test]
fn a_mistyped_override_fails_the_build() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("mistyped_override");
    std::fs::create_dir_all(&dir).expect("create case dir");
    std::fs::write(
        dir.join("glow.shader"),
        r"(public_inputs: [Float(1.0)], surface: (nodes: [], output: Lit((alpha: Input(0)))))",
    )
    .expect("write shader");
    let hsda = r#"[(attributes: (name: "p", material_graph: (
        path: "./glow.shader",
        overrides: {0: Vec3((0.0, 0.0, 0.0))},
    )))]"#;
    let input = dir.join("asset.hsda");
    std::fs::write(&input, hsda).expect("write source");

    let err = compile(&input).expect_err("should fail");
    assert!(err.to_string().contains("validating overrides"), "{err}");
}

/// Vertex-stage displacement compiles alongside a surface network, and a
/// leaf that only makes sense in the other network is rejected at build
/// time.
#[test]
fn a_displacement_graph_compiles() {
    let shader = r"(
        surface: (
            nodes: [],
            output: Unlit((color: Const(Color((1.0, 1.0, 1.0, 1.0))))),
        ),
        displacement: (
            nodes: [(kind: LocalNormal)],
            position_offset: Node(0),
        ),
    )";
    let package = compile(&write_source("displacement", shader)).expect("compile");
    let state = realize(&package);
    let prim = prim_named(&state, "p");
    let bytes = bulk_bytes(&package, prim, hsd::attributes::slots::MATERIAL_GRAPH_DATA);
    let graph = ShaderGraph::decode(bytes).expect("decode graph");
    assert!(graph.displacement.is_some());
}

/// A surface-only leaf (`WorldNormal`) inside the displacement network is a
/// build failure, not a silently-miscompiled shader — the vertex stage has
/// no fragment-stage varyings to read.
#[test]
fn a_wrong_network_leaf_fails_the_build() {
    let shader = r"(
        surface: (
            nodes: [],
            output: Unlit((color: Const(Color((1.0, 1.0, 1.0, 1.0))))),
        ),
        displacement: (
            nodes: [(kind: WorldNormal)],
            position_offset: Node(0),
        ),
    )";
    let err = compile(&write_source("wrong_network", shader)).expect_err("should fail");
    assert!(
        format!("{err:#}").contains("not legal outside its own network"),
        "{err:#}"
    );
}
