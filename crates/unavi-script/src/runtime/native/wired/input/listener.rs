use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::input::bindings::wired::input::{
        api::InputListener,
        types::{
            HostInputListener,
            InputEvent,
        },
    },
    shared,
};

impl HostInputListener for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<InputListener>,
    ) -> wasmtime::Result<Option<InputEvent>> {
        shared::wired::input::listener::poll(&self.api, self_.rep())
            .await
            .map(|event| event.map(Into::into))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<InputListener>) -> wasmtime::Result<()> {
        shared::wired::input::listener::drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}
