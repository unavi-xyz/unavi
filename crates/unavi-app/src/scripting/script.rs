use anyhow::{anyhow, Result};
use bevy::prelude::*;
use wasm_component_layer::{
    AsContextMut, Func, Instance, Linker, ResourceOwn, ResourceType, Store,
};

use super::{host::wired_ecs::EcsWorld, load::EngineBackend, util::blocking_lock, StoreData};

pub fn get_script_interface(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &Linker,
    instance: &Instance,
) -> Result<ScriptInterface> {
    let interface = instance
        .exports()
        .instance(&"wired:script/lifecycle".try_into()?)
        .ok_or(anyhow!("interface not found"))?;

    let data_type = interface
        .resource("data")
        .ok_or(anyhow!("data resource not found"))?;

    let init = interface.func("init").ok_or(anyhow!("init not found"))?;

    let update = interface
        .func("update")
        .ok_or(anyhow!("update not found"))?;

    let wired_ecs = linker
        .instance(&"wired:ecs/types".try_into()?)
        .ok_or(anyhow!("wired:ecs/types not found"))?;
    let ecs_world_type = wired_ecs
        .resource("ecs-world")
        .ok_or(anyhow!("ecs-world not found"))?;

    let resource_table = &store.data().resource_table.clone();
    let mut resource_table = blocking_lock(resource_table);
    let id = resource_table.next_id();

    let ecs_world = ResourceOwn::new(
        store.as_context_mut(),
        EcsWorld { id },
        ecs_world_type.clone(),
    )?;

    Ok(ScriptInterface {
        data_type,
        ecs_world,
        init,
        update,
    })
}

#[derive(Component)]
pub struct ScriptInterface {
    pub data_type: ResourceType,
    pub ecs_world: ResourceOwn,
    pub init: Func,
    pub update: Func,
}
