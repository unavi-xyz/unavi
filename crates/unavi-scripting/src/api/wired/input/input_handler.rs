use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender};
use wasm_bridge::component::Resource;

use crate::data::ScriptData;

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

#[derive(Default, Debug, Clone)]
pub struct InputHandlerRes(Arc<RwLock<InputHandlerData>>);

#[derive(Debug)]
pub struct InputHandlerData {
    pub sender: Sender<ScriptInputEvent>,
    receiver: Receiver<ScriptInputEvent>,
}

impl Default for InputHandlerData {
    fn default() -> Self {
        let (sender, receiver) = crossbeam::channel::bounded(10);
        Self { receiver, sender }
    }
}

impl InputHandlerRes {
    pub fn read(&self) -> RwLockReadGuard<InputHandlerData> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<InputHandlerData> {
        self.0.write().unwrap()
    }
}

impl HostInputHandler for ScriptData {
    fn new(&mut self) -> wasm_bridge::Result<Resource<InputHandlerRes>> {
        let handler = InputHandlerRes::default();
        let res = self.table.push(handler)?;
        Ok(res)
    }

    fn next(
        &mut self,
        self_: Resource<InputHandlerRes>,
    ) -> wasm_bridge::Result<Option<InputEvent>> {
        let data = self.table.get(&self_)?.0.read().unwrap();

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

    fn drop(&mut self, rep: Resource<InputHandlerRes>) -> wasm_bridge::Result<()> {
        Ok(())
    }
}
