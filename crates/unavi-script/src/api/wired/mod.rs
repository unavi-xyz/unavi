use wasmtime::component::{HasSelf, Linker};

use crate::{
    load::state::StoreState,
    permissions::{ApiName, ScriptPermissions},
};

pub mod agent;
pub mod scene;

pub fn add_to_linker(
    linker: &mut Linker<StoreState>,
    perms: &ScriptPermissions,
) -> anyhow::Result<()> {
    if perms.api.contains(&ApiName::Scene) {
        scene::bindings::wired::scene::context::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt.wired_scene
        })?;
        scene::bindings::wired::scene::types::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt.wired_scene
        })?;
    }
    if perms.api.contains(&ApiName::Agent) {
        agent::bindings::wired::agent::types::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt
        })?;
    }
    if perms.api.contains(&ApiName::LocalAgent) {
        agent::bindings::wired::agent::context::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt
        })?;
    }
    Ok(())
}
