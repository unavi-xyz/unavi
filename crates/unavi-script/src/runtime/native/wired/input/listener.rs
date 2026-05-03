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
        shared::wired::input::listener::poll(&self.backend, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<InputListener>) -> wasmtime::Result<()> {
        shared::wired::input::listener::drop(&self.backend, rep.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }
}
