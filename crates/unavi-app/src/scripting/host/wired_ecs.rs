use anyhow::Result;
use wasm_component_layer::{
    AsContextMut, Func, FuncType, Linker, ListType, ResourceOwn, ResourceType, Store, TupleType,
    Value, ValueType,
};

use crate::scripting::load::{EngineBackend, StoreData};

pub struct Component {}
pub struct ComponentInstance {}
pub struct EcsWorld {}
pub struct Entity {}
pub struct Query {}

pub enum WiredEcsCommand {
    RegisterComponent,
}

pub fn add_to_host(store: &mut Store<StoreData, EngineBackend>, linker: &mut Linker) -> Result<()> {
    // let (sender, reciever) = std::sync::mpsc::sync_channel(100);

    let component_instance_type = ResourceType::new::<ComponentInstance>(None);

    let component_type = ResourceType::new::<Component>(None);
    let component_new = {
        let component_instance_type = component_instance_type.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(component_type.clone())],
                [ValueType::Own(component_instance_type.clone())],
            ),
            move |ctx, _args, results| {
                results[0] = Value::Own(ResourceOwn::new(
                    ctx,
                    ComponentInstance {},
                    component_instance_type.clone(),
                )?);

                Ok(())
            },
        )
    };

    let entity_type = ResourceType::new::<Entity>(None);
    let entity_insert = Func::new(
        store.as_context_mut(),
        FuncType::new(
            [
                ValueType::Borrow(entity_type.clone()),
                ValueType::Own(component_instance_type.clone()),
            ],
            [],
        ),
        move |_ctx, _args, _results| {
            // TODO
            Ok(())
        },
    );

    let query_type = ResourceType::new::<Query>(None);
    let query_read = Func::new(
        store.as_context_mut(),
        FuncType::new(
            [ValueType::Borrow(query_type.clone())],
            [ValueType::List(ListType::new(ValueType::Tuple(
                TupleType::new(
                    None,
                    [
                        ValueType::Own(entity_type.clone()),
                        ValueType::List(ListType::new(ValueType::Own(
                            component_instance_type.clone(),
                        ))),
                    ],
                ),
            )))],
        ),
        move |_ctx, _args, _results| {
            // TODO
            Ok(())
        },
    );

    let ecs_world_type = ResourceType::new::<EcsWorld>(None);
    let ecs_world_register_component = {
        let component_type = component_type.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [ValueType::Borrow(ecs_world_type.clone())],
                [ValueType::Own(component_type.clone())],
            ),
            move |ctx, _args, results| {
                // sender.send(WiredEcsCommand::RegisterComponent)?;

                results[0] =
                    Value::Own(ResourceOwn::new(ctx, Component {}, component_type.clone())?);

                Ok(())
            },
        )
    };
    let ecs_world_register_query = {
        let query_type = query_type.clone();
        Func::new(
            store.as_context_mut(),
            FuncType::new(
                [
                    ValueType::Borrow(ecs_world_type.clone()),
                    ValueType::List(ListType::new(ValueType::Borrow(component_type.clone()))),
                ],
                [ValueType::Own(query_type.clone())],
            ),
            move |ctx, _args, results| {
                // let components = match &args[1] {
                //     Value::List(list) => list,
                //     _ => bail!("Wrong arg type."),
                // };

                results[0] = Value::Own(ResourceOwn::new(ctx, Query {}, query_type.clone())?);

                Ok(())
            },
        )
    };
    let ecs_world_spawn = Func::new(
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
        move |_ctx, _args, _results| {
            // TODO
            Ok(())
        },
    );

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

    Ok(())
}
