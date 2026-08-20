use std::future::Future;

use unavi_policy::document::ApiName;

use crate::runtime::{
    Runtime,
    shared,
};

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-peer",
        with: {
            "wired:error/types": crate::runtime::native::wired::error::bindings::wired::error::types,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use crate::runtime::native::wired::error::bindings::wired::error::types::Error;

impl bindings::wired::peer::types::Host for Runtime {}

impl bindings::wired::peer::api::Host for Runtime {
    fn self_peer(
        &mut self,
    ) -> impl Future<Output = wasmtime::Result<Result<Option<Vec<u8>>, Error>>> {
        std::future::ready(Ok(self
            .api
            .require(ApiName::Identity)
            .map(|()| shared::wired::peer::self_peer(&self.api))
            .map_err(Into::into)))
    }

    fn self_did(
        &mut self,
    ) -> impl Future<Output = wasmtime::Result<Result<Option<String>, Error>>> {
        std::future::ready(Ok(self
            .api
            .require(ApiName::Identity)
            .map(|()| shared::wired::peer::self_did(&self.api))
            .map_err(Into::into)))
    }

    fn doc_owner(
        &mut self,
        doc: Vec<u8>,
    ) -> impl Future<Output = wasmtime::Result<Result<Option<Vec<u8>>, Error>>> {
        std::future::ready(Ok(self
            .api
            .require(ApiName::Peer)
            .map(|()| shared::wired::peer::doc_owner(&self.api, doc))
            .map_err(Into::into)))
    }

    fn is_self_owner(&mut self) -> impl Future<Output = wasmtime::Result<Result<bool, Error>>> {
        std::future::ready(Ok(self
            .api
            .require(ApiName::Peer)
            .map(|()| shared::wired::peer::is_self_owner(&self.api))
            .map_err(Into::into)))
    }
}
