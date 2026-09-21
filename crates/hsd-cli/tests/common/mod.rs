#![allow(
    dead_code,
    reason = "compiled into every integration-test binary in this crate; not every binary uses every helper"
)]

use std::{
    collections::HashMap,
    path::Path,
};

use hsd::{
    attributes::name::NameAttr,
    id::PrimId,
    package::Package,
    state::{
        SceneState,
        entry::Entry,
    },
};
use hsd_cli::compile as compile_mod;

pub fn compile(input: &Path) -> anyhow::Result<Package> {
    compile_mod::compile_file(input, &mut HashMap::new())
}

/// The package is bytes on disk before it is state, so the test goes through
/// the encoding rather than around it.
pub fn realize(package: &Package) -> SceneState {
    let bytes = package.encode().expect("encode");
    let package = Package::decode(&bytes).expect("decode");

    let mut state = SceneState::new();
    for (key, value) in package.entries {
        state.apply(&Entry::new(key, value, 1)).expect("apply");
    }
    state
}

pub fn prim_named(state: &SceneState, name: &str) -> PrimId {
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
