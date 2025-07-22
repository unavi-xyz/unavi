use wasmtime::component::Linker;

use super::StoreState;

pub mod ecs;

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> anyhow::Result<()> {
    ecs::wired::ecs::host_api::add_to_linker::<StoreState, ecs::HasWiredEcsData>(linker, |s| {
        &mut s.data.wired_ecs
    })?;
    Ok(())
}
