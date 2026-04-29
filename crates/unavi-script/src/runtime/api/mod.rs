use wasmtime::component::Linker;

use crate::{
    permissions::{ApiName, ApiPermissions},
    runtime::StoreState,
};

pub mod wired;

pub fn add_apis_to_linker(
    _linker: &mut Linker<StoreState>,
    perms: &ApiPermissions,
) -> wasmtime::Result<()> {
    if perms.contains(&ApiName::Input) {
        // TODO
    }

    Ok(())
}
