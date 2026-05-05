use unavi_util::async_commands::AsyncCommands;

use crate::runtime::shared::{
    Api,
    slot_map::SlotMap,
    wired::input::{
        bridge::{GlobalInputListener, InputListener},
        listener::InputListenerRes,
    },
};

pub mod bridge;
pub mod listener;
pub mod types;

const INPUT_CHANNEL_LENGTH: usize = 8;

#[derive(Default)]
pub struct WiredInputApi {
    listeners: SlotMap<InputListenerRes>,
}

pub fn register_input_listener(backend: &Api, node: u32) -> anyhow::Result<u32> {
    let (target_doc, target_node) = backend
        .wired_scene
        .try_lock()?
        .nodes
        .get(node)
        .map(|n| (n.document, n.id))
        .ok_or_else(|| anyhow::anyhow!("node not found"))?;

    let (tx, rx) = async_channel::bounded(INPUT_CHANNEL_LENGTH);

    AsyncCommands::default()
        .spawn(InputListener {
            target_doc,
            target_node,
            tx,
        })
        .try_send()?;

    let rep = backend
        .wired_input
        .try_lock()?
        .listeners
        .insert(InputListenerRes { rx });

    Ok(rep)
}

pub fn register_global_input_listener(backend: &Api) -> anyhow::Result<u32> {
    let (tx, rx) = async_channel::bounded(INPUT_CHANNEL_LENGTH);

    AsyncCommands::default()
        .spawn(GlobalInputListener { tx })
        .try_send()?;

    let rep = backend
        .wired_input
        .try_lock()?
        .listeners
        .insert(InputListenerRes { rx });

    Ok(rep)
}
