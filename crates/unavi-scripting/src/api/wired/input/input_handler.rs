use std::cell::Cell;

use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender};
use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefCountCell, RefResource},
    data::StoreData,
};

use super::bindings::{
    handler::{HostInputHandler, InputEvent},
    types::{InputAction, InputData, Ray},
};

pub enum ScriptInputEvent {
    Raycast {
        action: InputAction,
        origin: bevy::math::Vec3,
        orientation: bevy::math::Quat,
    },
}

#[derive(Component, Deref)]
pub struct InputHandlerSender(pub Sender<ScriptInputEvent>);

#[derive(Debug)]
pub struct InputHandler {
    pub sender: Sender<ScriptInputEvent>,
    receiver: Receiver<ScriptInputEvent>,
    ref_count: RefCountCell,
}

impl RefCount for InputHandler {
    fn ref_count(&self) -> &Cell<usize> {
        &self.ref_count
    }
}

impl RefResource for InputHandler {}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    pub fn new() -> Self {
        let (sender, receiver) = crossbeam::channel::bounded(10);

        Self {
            receiver,
            ref_count: RefCountCell::default(),
            sender,
        }
    }
}

impl HostInputHandler for StoreData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<InputHandler>> {
        let handler = InputHandler::new();
        let table_res = self.table.push(handler)?;
        let res = InputHandler::from_res(&table_res, &self.table)?;
        Ok(res)
    }

    fn next(&mut self, self_: Resource<InputHandler>) -> wasm_bridge::Result<Option<InputEvent>> {
        let data = self.table.get(&self_)?;

        if let Ok(event) = data.receiver.try_recv() {
            let e = match event {
                ScriptInputEvent::Raycast {
                    action,
                    origin,
                    orientation,
                } => InputEvent {
                    id: 0, // TODO: Set id
                    action,
                    data: InputData::Ray(Ray {
                        origin: origin.into(),
                        orientation: orientation.into(),
                    }),
                },
            };

            Ok(Some(e))
        } else {
            Ok(None)
        }
    }

    fn drop(&mut self, rep: Resource<InputHandler>) -> wasm_bridge::Result<()> {
        InputHandler::handle_drop(rep, &mut self.table)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::api::utils::tests::init_test_data;

    use super::*;

    #[test]
    #[traced_test]
    fn test_drop() {
        let (_, mut data) = init_test_data();

        let res = HostInputHandler::new(&mut data).unwrap();

        crate::api::utils::tests::test_drop(&mut data, res);
    }

    #[test]
    #[traced_test]
    fn test_new() {
        let (_, mut data) = init_test_data();

        let res = HostInputHandler::new(&mut data).unwrap();

        crate::api::utils::tests::test_new(&mut data, res);
    }
}
