use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    native::wired::input::bindings::wired::input::{api::InputListener, types::HostInputListener},
};

impl HostInputListener for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<InputListener>,
    ) -> wasmtime::Result<Option<super::bindings::wired::input::types::InputEvent>> {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<InputListener>) -> wasmtime::Result<()> {
        todo!()
    }
}
