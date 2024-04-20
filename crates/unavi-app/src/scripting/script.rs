use anyhow::{anyhow, Result};
use wasm_component_layer::{
    ComponentType, Instance, ResourceOwn, ResourceType, TypedFunc, UnaryComponentType, Value,
    ValueType,
};
use wasmtime::component::Resource;

use super::host::wired_ecs::EcsWorld;

pub fn get_script_interface(instance: &Instance) -> Result<ScriptInterface> {
    let interface = instance
        .exports()
        .instance(&"wired:script/script".try_into()?)
        .ok_or(anyhow!("interface not found"))?;

    let init = interface
        .func("init")
        .ok_or(anyhow!("init not found"))?
        .typed()?;

    Ok(ScriptInterface { init })
}

pub struct ScriptInterface {
    init: TypedFunc<Resource<EcsWorld>, ()>,
}

pub struct ScriptData {}
