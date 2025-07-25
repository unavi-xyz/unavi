use wasmtime::component::Linker;

use crate::load::state::StoreState;

pub mod ecs;

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> anyhow::Result<()> {
    ecs::wired::ecs::host_api::add_to_linker::<StoreState, ecs::HasWiredEcsData>(linker, |s| {
        &mut s.rt.wired_ecs
    })?;
    Ok(())
}
