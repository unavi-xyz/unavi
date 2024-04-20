use bevy::tasks::futures_lite::future;
use wasm_bridge::component::Resource;

use crate::scripting::script::{
    commands::{
        ComponentInstance as ComponentInstanceState, RegisterQuery, ScriptCommand, SpawnEntity,
    },
    state::ScriptState,
};

#[derive(Debug)]
pub struct ComponentInstance {
    component: u32,
}

#[derive(Debug)]
pub struct Component;

#[derive(Debug)]
pub struct EcsWorld;

#[derive(Debug)]
pub struct Entity;

type QueryResultIds = Vec<(u32, Vec<u32>)>;
type QueryResult = Vec<(Resource<Entity>, Vec<Resource<ComponentInstance>>)>;

#[derive(Debug)]
pub struct Query {
    components: Vec<u32>,
    pub result: QueryResultIds,
}

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-ecs",
    tracing: true,
    with: {
        "wired:ecs/types/component": Component,
        "wired:ecs/types/component-instance": ComponentInstance,
        "wired:ecs/types/ecs-world": EcsWorld,
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

        let mut table = future::block_on(async { self.table.lock().await });
        let resource = table.push_child(instance, &self_)?;
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
    fn read(&mut self, self_: Resource<Query>) -> wasm_bridge::Result<QueryResult> {
        let table = future::block_on(async { self.table.lock().await });
        let query = table.get(&self_)?;

        Ok(query
            .result
            .iter()
            .map(|(entity, components)| {
                (
                    Resource::new_own(*entity),
                    components.iter().map(|c| Resource::new_own(*c)).collect(),
                )
            })
            .collect())
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
        let mut table = future::block_on(async { self.table.lock().await });

        let components = components
            .iter()
            .map(|r| -> anyhow::Result<ComponentInstanceState> {
                let instance = table.get(r)?;
                Ok(ComponentInstanceState {
                    component: instance.component,
                    instance: r.rep() as u64,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let entity = table.push(Entity)?;

        let sender = future::block_on(async { self.sender.lock().await });
        sender.send(ScriptCommand::SpawnEntity(SpawnEntity {
            id: entity.rep(),
            components,
        }))?;

        Ok(entity)
    }

    fn register_component(
        &mut self,
        _self_: Resource<EcsWorld>,
    ) -> wasm_bridge::Result<Resource<Component>> {
        let mut table = future::block_on(async { self.table.lock().await });
        let resource = table.push(Component)?;

        let sender = future::block_on(async { self.sender.lock().await });
        sender.send(ScriptCommand::RegisterComponent(resource.rep()))?;

        Ok(resource)
    }

    fn register_query(
        &mut self,
        _self_: Resource<EcsWorld>,
        components: Vec<Resource<Component>>,
    ) -> wasm_bridge::Result<Resource<Query>> {
        let components = components.iter().map(|r| r.rep()).collect::<Vec<_>>();
        let query = Query {
            components: components.clone(),
            result: Default::default(),
        };

        let mut table = future::block_on(async { self.table.lock().await });
        let resource = table.push(query)?;

        let sender = future::block_on(async { self.sender.lock().await });
        sender.send(ScriptCommand::RegisterQuery(RegisterQuery {
            id: resource.rep(),
            components,
        }))?;

        Ok(resource)
    }

    fn drop(&mut self, _rep: Resource<EcsWorld>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}

impl wired::ecs::types::Host for ScriptState {}
