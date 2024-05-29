use anyhow::{anyhow, Result};
use bevy::prelude::*;
use wasm_component_layer::{Func, Instance, ResourceType};

pub fn get_script_interface(instance: &Instance) -> Result<ScriptInterface> {
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

    Ok(ScriptInterface {
        data_type,
        init,
        update,
    })
}

#[derive(Component)]
pub struct ScriptInterface {
    pub data_type: ResourceType,
    pub init: Func,
    pub update: Func,
}
