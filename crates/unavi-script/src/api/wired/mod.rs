use wasmtime::component::{HasSelf, Linker};

use crate::{load::state::StoreState, permissions::ScriptPermissions};

pub mod agent;
pub mod scene;

pub fn add_to_linker(
    linker: &mut Linker<StoreState>,
    perms: &ScriptPermissions,
) -> anyhow::Result<()> {
    if perms.wired_scene {
        scene::bindings::wired::scene::context::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt.wired_scene
        })?;
        scene::bindings::wired::scene::types::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt.wired_scene
        })?;
    }
    if perms.wired_local_agent || perms.wired_agent {
        agent::bindings::wired::agent::types::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt
        })?;
    }
    if perms.wired_local_agent {
        agent::bindings::wired::agent::context::add_to_linker::<_, HasSelf<_>>(linker, |s| {
            &mut s.rt
        })?;
    }
    Ok(())
}
