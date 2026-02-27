use wasmtime::component::{HasSelf, Linker};

use crate::load::state::StoreState;

pub mod scene;

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> anyhow::Result<()> {
    scene::bindings::wired::scene::types::add_to_linker::<_, HasSelf<_>>(linker, |s| {
        &mut s.rt.wired_scene
    })?;

    Ok(())
}
