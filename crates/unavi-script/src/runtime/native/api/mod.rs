use wasmtime::component::Linker;

use crate::{
    permissions::{ApiName, ApiPermissions},
    runtime::Runtime,
};

pub mod wired;

pub fn add_apis_to_linker(
    _linker: &mut Linker<Runtime>,
    perms: &ApiPermissions,
) -> wasmtime::Result<()> {
    if perms.contains(&ApiName::Input) {
        // TODO
    }

    Ok(())
}
