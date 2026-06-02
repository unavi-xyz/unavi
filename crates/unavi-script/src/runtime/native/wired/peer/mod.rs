use crate::runtime::{
    Runtime,
    shared,
};

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-peer",
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

impl bindings::wired::peer::types::Host for Runtime {}

impl bindings::wired::peer::api::Host for Runtime {
    async fn self_peer(&mut self) -> wasmtime::Result<Vec<u8>> {
        Ok(shared::wired::peer::self_peer(&self.api))
    }

    async fn doc_owner(&mut self, doc: Vec<u8>) -> wasmtime::Result<Option<Vec<u8>>> {
        Ok(shared::wired::peer::doc_owner(&self.api, doc))
    }

    async fn is_self_owner(&mut self) -> wasmtime::Result<bool> {
        Ok(shared::wired::peer::is_self_owner(&self.api))
    }
}
