use anyhow::Result;
use crossbeam::channel::Sender;
use wasm_component_layer::{
    AsContextMut, Func, FuncType, Linker, List, ListType, ResourceType, Store, Value, ValueType,
};

use crate::{load::EngineBackend, StoreData};

use super::WiredGltfAction;

#[derive(Clone)]
pub struct NodeResource {
    pub id: u32,
}

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
    sender: Sender<WiredGltfAction>,
) -> Result<()> {
    let interface = linker.define_instance("wired:gltf/node".try_into()?)?;

    let resource_table = store.data().resource_table.clone();

    let node_type = ResourceType::new::<NodeResource>(None);
    let nodes_list_type = ListType::new(ValueType::Own(node_type.clone()));

    // TODO:
    //  - actions
    //      - send create / remove messages over channel to bevy system
    //      - read messages and apply them to ECS
    //  - queries
    //      - query node transform data each frame, copy it into Mutex
    //      - use Mutex here for nodes query
    //  - for other data, like names, we can maintain a version here, and copy it into ECS when
    //  changed

    let nodes = Func::new(
        store.as_context_mut(),
        FuncType::new([], [ValueType::List(nodes_list_type.clone())]),
        move |_ctx, _args, results| {
            results[0] = Value::List(List::new(nodes_list_type.clone(), []).unwrap());

            Ok(())
        },
    );

    let create_node = Func::new(
        store.as_context_mut(),
        FuncType::new([], [ValueType::Own(node_type.clone())]),
        move |mut ctx, _args, results| {
            let mut resource_table = resource_table.write().unwrap();

            let (id, resource) =
                resource_table.push(ctx.as_context_mut(), node_type.clone(), |id| {
                    NodeResource { id }
                })?;

            sender.send(WiredGltfAction::CreateNode { id })?;

            results[0] = Value::Own(resource);

            Ok(())
        },
    );

    interface.define_func("nodes", nodes)?;
    interface.define_func("create-node", create_node)?;

    Ok(())
}
