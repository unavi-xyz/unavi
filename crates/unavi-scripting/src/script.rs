use anyhow::{anyhow, Result};
use bevy::ecs::component::Component;
use wasm_component_layer::{Func, Instance};

pub fn get_script_interface(instance: &Instance) -> Result<ScriptInterface> {
    let interface = instance
        .exports()
        .instance(&"wired:script/types".try_into()?)
        .ok_or(anyhow!("interface not found"))?;

    let construct = interface
        .func("[constructor]script")
        .ok_or(anyhow!("construct not found"))?;

    let update = interface
        .func("[method]script.update")
        .ok_or(anyhow!("update not found"))?;

    Ok(ScriptInterface { construct, update })
}

#[derive(Component)]
pub struct ScriptInterface {
    pub construct: Func,
    pub update: Func,
}
