use wasm_bridge::component::Resource;

use crate::scripting::script::state::ScriptState;

pub use self::wired::ecs::types::{Component, EcsWorld, Entity};

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-ecs",
    tracing: true
});

impl wired::ecs::types::HostEntity for ScriptState {
    fn drop(&mut self, _rep: Resource<Entity>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl wired::ecs::types::HostComponent for ScriptState {
    fn drop(&mut self, _rep: Resource<Component>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl wired::ecs::types::HostEcsWorld for ScriptState {
    fn spawn(&mut self, _self_: Resource<EcsWorld>) -> wasm_bridge::Result<Resource<Entity>> {
        Ok(Resource::new_own(0))
    }

    fn register_component(
        &mut self,
        _self_: Resource<wired::ecs::types::EcsWorld>,
    ) -> wasm_bridge::Result<Resource<Component>> {
        Ok(Resource::new_own(0))
    }

    fn drop(&mut self, _rep: Resource<EcsWorld>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl wired::ecs::types::Host for ScriptState {}
