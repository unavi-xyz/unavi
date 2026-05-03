use anyhow::bail;
use unavi_util::async_commands::AsyncCommands;

use crate::runtime::shared::{
    RuntimeBackend,
    slot_map::SlotMap,
    wired::input::{bridge::InputListener, listener::InputListenerRes},
};

pub(crate) mod bridge;
pub mod listener;

#[derive(Default)]
pub struct WiredInputBackend {
    listeners: SlotMap<InputListenerRes>,
}

pub async fn register_input_listener(
    backend: &mut RuntimeBackend,
    node: u32,
) -> anyhow::Result<u32> {
    let lock = backend.wired_scene.lock().await;

    let Some(node_res) = lock.nodes.get(node) else {
        bail!("node not found")
    };

    let (tx, rx) = async_channel::bounded(8);

    let entity = AsyncCommands::default()
        .send_spawn(InputListener {
            tx,
            target_doc: node_res.doc_id,
            target_node: node_res.id,
        })
        .await;

    let rep = backend
        .wired_input
        .try_lock()
        .expect("lock")
        .listeners
        .insert(InputListenerRes { node, entity, rx });

    Ok(rep)
}
