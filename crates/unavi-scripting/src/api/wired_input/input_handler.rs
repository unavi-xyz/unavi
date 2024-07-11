use std::cell::Cell;

use bevy::prelude::{Component, Deref};
use crossbeam::channel::{Receiver, Sender};
use wasm_bridge::component::Resource;

use crate::{
    api::utils::{RefCount, RefCountCell, RefResource},
    state::StoreState,
};

use super::wired::{
    input::{
        handler::HostInputHandler,
        types::{InputEvent, InputType, Ray},
    },
    math::types::{Quat, Vec3},
};

pub enum ScriptInputEvent {
    Raycast {
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

impl HostInputHandler for StoreState {
    fn new(&mut self) -> wasm_bridge::Result<Resource<InputHandler>> {
        let handler = InputHandler::new();
        let table_res = self.table.push(handler)?;
        let res = InputHandler::from_res(&table_res, &self.table)?;
        Ok(res)
    }

    fn handle_input(
        &mut self,
        self_: Resource<InputHandler>,
    ) -> wasm_bridge::Result<Option<InputEvent>> {
        let data = self.table.get(&self_)?;

        if let Ok(event) = data.receiver.try_recv() {
            let e = match event {
                ScriptInputEvent::Raycast {
                    origin,
                    orientation,
                } => {
                    let origin = Vec3 {
                        x: origin.x,
                        y: origin.y,
                        z: origin.z,
                    };

                    let orientation = Quat {
                        x: orientation.x,
                        y: orientation.y,
                        z: orientation.z,
                        w: orientation.w,
                    };

                    InputEvent {
                        id: 0,
                        input: InputType::Ray(Ray {
                            origin,
                            orientation,
                        }),
                        order: 0,
                        distance: 0.0,
                    }
                }
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
    use bevy::prelude::*;
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_drop() {
        let mut world = World::default();
        let ent = world.spawn(()).id();
        let mut state = StoreState::new("test_drop".to_string(), ent);

        let res = HostInputHandler::new(&mut state).unwrap();

        crate::api::utils::tests::test_drop(&mut state, res);
    }

    #[test]
    #[traced_test]
    fn test_new() {
        let mut world = World::default();
        let ent = world.spawn(()).id();
        let mut state = StoreState::new("test_new".to_string(), ent);

        let res = HostInputHandler::new(&mut state).unwrap();

        crate::api::utils::tests::test_new(&mut state, res);
    }
}
