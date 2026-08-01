use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::input::bindings::wired::input::{
        api::InputListener,
        types::{
            HostInputListener,
            InputAction,
            InputDevice,
            InputEvent,
        },
    },
    shared::{
        self,
        wired::input::types as shared_types,
    },
};

impl From<shared_types::InputAction> for InputAction {
    fn from(a: shared_types::InputAction) -> Self {
        match a {
            shared_types::InputAction::GrabDown => Self::GrabDown,
            shared_types::InputAction::GrabUp => Self::GrabUp,
            shared_types::InputAction::MenuDown => Self::MenuDown,
            shared_types::InputAction::MenuUp => Self::MenuUp,
            shared_types::InputAction::ScrollUp => Self::ScrollUp,
            shared_types::InputAction::ScrollDown => Self::ScrollDown,
        }
    }
}

impl From<shared_types::InputDevice> for InputDevice {
    fn from(d: shared_types::InputDevice) -> Self {
        match d {
            shared_types::InputDevice::Keyboard => Self::Keyboard,
            shared_types::InputDevice::LeftHand => Self::LeftHand,
            shared_types::InputDevice::RightHand => Self::RightHand,
        }
    }
}

impl From<shared_types::InputEvent> for InputEvent {
    fn from(e: shared_types::InputEvent) -> Self {
        Self {
            action: e.action.into(),
            device: e.device.into(),
        }
    }
}

impl HostInputListener for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<InputListener>,
    ) -> wasmtime::Result<Option<InputEvent>> {
        shared::wired::input::listener::poll(&self.api, self_.rep())
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<InputListener>) -> wasmtime::Result<()> {
        shared::wired::input::listener::drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}
