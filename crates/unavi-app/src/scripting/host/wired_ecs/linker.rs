use std::sync::Arc;

use anyhow::{bail, Result};
use wasm_component_layer::{
    AsContext, AsContextMut, Func, FuncType, Linker, List, ListType, ResourceOwn, ResourceType,
    Store, TupleType, Value, ValueType,
};

use crate::scripting::{load::EngineBackend, util::blocking_lock, StoreData};

use super::{WiredEcsCommand, WiredEcsReceiver};

pub struct Component {
    pub id: u32,
}

#[derive(Clone)]
pub struct ComponentInstance {
    pub id: u32,
    pub component: u32,
}

pub struct EcsWorld {
    pub id: u32,
}

pub struct Entity {
    pub id: u32,
}

pub struct Query {
    id: u32,
}

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
) -> Result<WiredEcsReceiver> {
    let resource_table = &store.data().resource_table.clone();

    let (sender, receiver) = crossbeam::channel::bounded(256);
    let sender = Arc::new(sender);

    let component_instance_type = ResourceType::new::<ComponentInstance>(None);

    let component_type = ResourceType::new::<Component>(None);
    let component_new = {
        let component_instance_type = component_instance_type.clone();
        let resource_table = resource_table.clone();
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(component_type.clone())],
                [ValueType::Own(component_instance_type.clone())],
            ),
            move |mut ctx, args, results| {
                let component = match &args[0] {
                    Value::Borrow(r) => r,
                    _ => bail!("invalid arg type"),
                };

                let ctx_ref = ctx.as_context();
                let component_rep: &Component = component.rep(&ctx_ref)?;
                let component = component_rep.id;

                let mut resource_table = blocking_lock(&resource_table);
                let id = resource_table.next_id();

                let resource = ResourceOwn::new(
                    ctx.as_context_mut(),
                    ComponentInstance { id, component },
                    component_instance_type.clone(),
                )?;

                sender.send(WiredEcsCommand::RegisterComponent { id })?;

                results[0] = Value::Own(resource);

                Ok(())
            },
        )
    };

    let entity_type = ResourceType::new::<Entity>(None);
    let entity_insert = {
        let sender = sender.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(entity_type.clone()),
                    ValueType::Own(component_instance_type.clone()),
                ],
                [],
            ),
            move |ctx, args, _results| {
                let entity = match &args[0] {
                    Value::Borrow(r) => r,
                    _ => bail!("invalid arg type"),
                };

                let ctx = ctx.as_context();
                let entity_rep: &Entity = entity.rep(&ctx)?;

                let instance = match &args[1] {
                    Value::Own(r) => r,
                    _ => bail!("invalid arg type"),
                };
                let instance_rep: &ComponentInstance = instance.rep(&ctx)?;

                sender.send(WiredEcsCommand::Insert {
                    entity: entity_rep.id,
                    instance: instance_rep.id,
                })?;

                Ok(())
            },
        )
    };

    let query_type = ResourceType::new::<Query>(None);
    let query_result_type = ListType::new(ValueType::Tuple(TupleType::new(
        None,
        [
            ValueType::Own(entity_type.clone()),
            ValueType::List(ListType::new(ValueType::Own(
                component_instance_type.clone(),
            ))),
        ],
    )));
    let query_read = {
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(query_type.clone())],
                [ValueType::List(query_result_type.clone())],
            ),
            move |_ctx, _args, results| {
                results[0] = Value::List(List::new(query_result_type.clone(), Vec::new())?);

                Ok(())
            },
        )
    };

    let ecs_world_type = ResourceType::new::<EcsWorld>(None);
    let ecs_world_register_component = {
        let component_type = component_type.clone();
        let sender = sender.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(ecs_world_type.clone())],
                [ValueType::Own(component_type.clone())],
            ),
            move |mut ctx, _args, results| {
                let mut resource_table = blocking_lock(&resource_table);
                let id = resource_table.next_id();

                let resource = ResourceOwn::new(
                    ctx.as_context_mut(),
                    Component { id },
                    component_type.clone(),
                )?;

                sender.send(WiredEcsCommand::RegisterComponent { id })?;

                results[0] = Value::Own(resource);

                Ok(())
            },
        )
    };
    let ecs_world_register_query = {
        let query_type = query_type.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(ecs_world_type.clone()),
                    ValueType::List(ListType::new(ValueType::Borrow(component_type.clone()))),
                ],
                [ValueType::Own(query_type.clone())],
            ),
            move |ctx, args, results| {
                let _components = match &args[1] {
                    Value::List(list) => list,
                    _ => bail!("invalid arg type"),
                };

                let mut resource_table = blocking_lock(&resource_table);
                let id = resource_table.next_id();

                let resource = ResourceOwn::new(ctx, Query { id }, query_type.clone())?;

                results[0] = Value::Own(resource);

                Ok(())
            },
        )
    };
    let ecs_world_spawn = {
        let entity_type = entity_type.clone();
        let sender = sender.clone();
        let resource_table = resource_table.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(ecs_world_type.clone()),
                    ValueType::List(ListType::new(ValueType::Own(
                        component_instance_type.clone(),
                    ))),
                ],
                [ValueType::Own(entity_type.clone())],
            ),
            move |ctx, args, results| {
                let instances = match &args[1] {
                    Value::List(list) => list,
                    _ => bail!("invalid arg type"),
                };

                let components = instances
                    .into_iter()
                    .map(|value| {
                        let resource = match &value {
                            Value::Own(r) => r,
                            _ => bail!("invalid arg type"),
                        };

                        let ctx = ctx.as_context();
                        resource.rep(&ctx).cloned()
                    })
                    .collect::<Result<Vec<_>>>()?;

                let mut resource_table = blocking_lock(&resource_table);
                let id = resource_table.next_id();

                sender.send(WiredEcsCommand::Spawn { id, components })?;

                let resource = ResourceOwn::new(ctx, Entity { id }, entity_type.clone())?;
                results[0] = Value::Own(resource);

                Ok(())
            },
        )
    };

    let interface = linker.define_instance("wired:ecs/types".try_into()?)?;

    interface.define_resource("component-instance", component_instance_type)?;

    interface.define_resource("component", component_type)?;
    interface.define_func("[method]component.new", component_new)?;

    interface.define_resource("entity", entity_type)?;
    interface.define_func("[method]entity.insert", entity_insert)?;

    interface.define_resource("query", query_type)?;
    interface.define_func("[method]query.read", query_read)?;

    interface.define_resource("ecs-world", ecs_world_type)?;
    interface.define_func(
        "[method]ecs-world.register-component",
        ecs_world_register_component,
    )?;
    interface.define_func("[method]ecs-world.register-query", ecs_world_register_query)?;
    interface.define_func("[method]ecs-world.spawn", ecs_world_spawn)?;

    Ok(WiredEcsReceiver(receiver))
}
