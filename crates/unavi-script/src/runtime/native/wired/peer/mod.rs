use crate::{
    permissions::ApiName,
    runtime::{
        Runtime,
        shared,
    },
};

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-peer",
        with: {
            "wired:error/types": crate::runtime::native::wired::error::types,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use crate::runtime::native::wired::error::Error;

impl bindings::wired::peer::types::Host for Runtime {}

impl bindings::wired::peer::api::Host for Runtime {
    async fn self_peer(&mut self) -> wasmtime::Result<Result<Option<Vec<u8>>, Error>> {
        Ok(self
            .api
            .require(ApiName::Peer)
            .map(|()| shared::wired::peer::self_peer(&self.api))
            .map_err(Into::into))
    }

    async fn doc_owner(
        &mut self,
        doc: Vec<u8>,
    ) -> wasmtime::Result<Result<Option<Vec<u8>>, Error>> {
        Ok(self
            .api
            .require(ApiName::Peer)
            .map(|()| shared::wired::peer::doc_owner(&self.api, doc))
            .map_err(Into::into))
    }

    async fn is_self_owner(&mut self) -> wasmtime::Result<Result<bool, Error>> {
        Ok(self
            .api
            .require(ApiName::Peer)
            .map(|()| shared::wired::peer::is_self_owner(&self.api))
            .map_err(Into::into))
    }
}
