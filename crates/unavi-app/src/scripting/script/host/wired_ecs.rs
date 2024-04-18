use wasm_bridge::component::Resource;
use wasm_bridge_wasi::ResourceTableError;

use crate::scripting::script::{
    commands::{ComponentInstance as ComponentInstanceState, ScriptCommand, SpawnEntity},
    state::ScriptState,
};

pub use self::wired::ecs::types::EcsWorld;

pub struct ComponentInstance {
    component: u32,
}

pub struct Component;

pub struct Entity;

pub struct Query {
    components: Vec<u32>,
}

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-ecs",
    tracing: true,
    with: {
        "wired:ecs/types/component-instance": ComponentInstance,
        "wired:ecs/types/component": Component,
        "wired:ecs/types/entity": Entity,
        "wired:ecs/types/query": Query,
    }
});

impl wired::ecs::types::HostComponentInstance for ScriptState {
    fn drop(&mut self, _rep: Resource<ComponentInstance>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl wired::ecs::types::HostComponent for ScriptState {
    fn new(
        &mut self,
        self_: Resource<Component>,
    ) -> wasm_bridge::Result<Resource<ComponentInstance>> {
        let instance = ComponentInstance {
            component: self_.rep(),
        };
        let resource = self.table.push_child(instance, &self_)?;
        Ok(resource)
    }

    fn drop(&mut self, _rep: Resource<Component>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl wired::ecs::types::HostEntity for ScriptState {
    fn insert(
        &mut self,
        _self_: Resource<Entity>,
        _component: Resource<ComponentInstance>,
    ) -> wasm_bridge::Result<()> {
        // TODO: command
        Ok(())
    }

    fn drop(&mut self, _rep: Resource<Entity>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl wired::ecs::types::HostQuery for ScriptState {
    fn read(
        &mut self,
        _self_: Resource<Query>,
    ) -> wasm_bridge::Result<Vec<(Resource<Entity>, Resource<ComponentInstance>)>> {
        // TODO
        Ok(Vec::new())
    }

    fn drop(&mut self, _rep: Resource<Query>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl wired::ecs::types::HostEcsWorld for ScriptState {
    fn spawn(
        &mut self,
        _self_: Resource<EcsWorld>,
        components: Vec<Resource<ComponentInstance>>,
    ) -> wasm_bridge::Result<Resource<Entity>> {
        let components = components
            .iter()
            .map(|r| -> Result<ComponentInstanceState, ResourceTableError> {
                let instance = self.table.get(r)?;
                Ok(ComponentInstanceState {
                    component: instance.component,
                    instance: r.rep(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let entity = self.table.push(Entity)?;

        self.sender.send(ScriptCommand::SpawnEntity(SpawnEntity {
            id: entity.rep(),
            components,
        }))?;

        Ok(entity)
    }

    fn register_component(
        &mut self,
        _self_: Resource<wired::ecs::types::EcsWorld>,
    ) -> wasm_bridge::Result<Resource<Component>> {
        let resource = self.table.push(Component)?;

        self.sender
            .send(ScriptCommand::RegisterComponent(resource.rep()))?;

        Ok(resource)
    }

    fn register_query(
        &mut self,
        _self_: Resource<EcsWorld>,
        components: Vec<Resource<Component>>,
    ) -> wasm_bridge::Result<Resource<Query>> {
        let components = components.iter().map(|r| r.rep()).collect();
        let query = Query { components };
        let resource = self.table.push(query)?;

        // TODO: command

        Ok(resource)
    }

    fn drop(&mut self, _rep: Resource<EcsWorld>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl wired::ecs::types::Host for ScriptState {}
