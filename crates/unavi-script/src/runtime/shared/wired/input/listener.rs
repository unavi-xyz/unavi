use async_channel::Receiver;
use bevy::ecs::entity::Entity;

use crate::runtime::native::wired::input::bindings::wired::input::types::InputEvent;

pub struct InputListenerRes {
    pub node: u32,
    pub entity: Entity,
    pub rx: Receiver<InputEvent>,
}

pub fn poll(listener: u32) {}
