use unavi_input::pointer::PointerKind;
use unavi_util::async_commands::AsyncCommands;

use crate::runtime::shared::{
    Api,
    registry::pointer::POINTER_REGISTRY,
    slot_map::SlotMap,
    wired::input::{
        bridge::{
            GlobalInputListener,
            InputListener,
        },
        listener::{
            InputListenerRes,
            InputQueue,
        },
        types::Pointer,
    },
};

pub mod bridge;
pub mod listener;
pub mod types;

#[derive(Default)]
pub struct WiredInputApi {
    listeners: SlotMap<InputListenerRes>,
}

pub async fn register_input_listener(backend: &Api, node: u32) -> anyhow::Result<u32> {
    let (target_doc, target_prim) = backend
        .wired_scene
        .lock()
        .await
        .prims
        .get(node)
        .map(|prim| (prim.doc_id, prim.id))
        .ok_or_else(|| anyhow::anyhow!("node not found"))?;

    let queue = InputQueue::default();

    AsyncCommands::default()
        .spawn(InputListener {
            target_doc,
            target_prim,
            queue: queue.clone(),
        })
        .send()
        .await?;

    let rep = backend
        .wired_input
        .lock()
        .await
        .listeners
        .insert(InputListenerRes { queue }, &backend.quota)?;

    Ok(rep)
}

pub async fn register_global_input_listener(backend: &Api) -> anyhow::Result<u32> {
    let queue = InputQueue::default();

    AsyncCommands::default()
        .spawn(GlobalInputListener {
            queue: queue.clone(),
        })
        .send()
        .await?;

    let rep = backend
        .wired_input
        .lock()
        .await
        .listeners
        .insert(InputListenerRes { queue }, &backend.quota)?;

    Ok(rep)
}

#[must_use]
pub fn pointers() -> Vec<Pointer> {
    let snapshot = POINTER_REGISTRY.read();
    PointerKind::ALL
        .into_iter()
        .map(|kind| snapshot[kind.index()].unwrap_or_else(|| Pointer::inactive(kind)))
        .collect()
}
