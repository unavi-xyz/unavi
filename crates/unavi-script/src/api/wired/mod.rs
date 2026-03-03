use wasmtime::component::{HasSelf, Linker};

use crate::{load::state::StoreState, permissions::ScriptPermissions};

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
    Ok(())
}
