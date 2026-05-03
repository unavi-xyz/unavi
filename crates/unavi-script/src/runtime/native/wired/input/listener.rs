use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::input::bindings::wired::input::{
        api::InputListener,
        types::{HostInputListener, InputEvent},
    },
    shared,
};

impl HostInputListener for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<InputListener>,
    ) -> wasmtime::Result<Option<InputEvent>> {
        shared::wired::input::listener::poll(self_.rep());
        todo!()
    }

    async fn drop(&mut self, rep: Resource<InputListener>) -> wasmtime::Result<()> {
        todo!()
    }
}
