use std::future::Future;

use unavi_policy::document::ApiName;
use wasmtime::component::Resource;

use crate::{
    error::ScriptError,
    runtime::{
        Runtime,
        shared::{
            self,
            wired::storage::{
                GetFutureRes,
                ListFutureRes,
                StorageRes,
            },
        },
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::storage::{
        GetFutureRes,
        ListFutureRes,
        StorageRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-storage",
        with: {
            "wired:storage/types.storage":     StorageRes,
            "wired:storage/types.get-future":  GetFutureRes,
            "wired:storage/types.list-future": ListFutureRes,
            "wired:error/types": crate::runtime::native::wired::error::bindings::wired::error::types,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::storage::types::{
    Entry as WitEntry,
    HostGetFuture,
    HostListFuture,
    HostStorage,
    Storage,
};

use crate::runtime::native::wired::error::bindings::wired::error::types::Error;

impl bindings::wired::storage::types::Host for Runtime {}

impl HostStorage for Runtime {
    async fn get(
        &mut self,
        self_: Resource<StorageRes>,
        ns: Vec<u8>,
        key: String,
    ) -> wasmtime::Result<Resource<GetFutureRes>> {
        shared::wired::storage::get(&self.api, self_.rep(), ns, key)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn list(
        &mut self,
        self_: Resource<StorageRes>,
        ns: Vec<u8>,
        prefix: String,
    ) -> wasmtime::Result<Resource<ListFutureRes>> {
        shared::wired::storage::list(&self.api, self_.rep(), ns, prefix)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    fn root_doc(
        &mut self,
        self_: Resource<StorageRes>,
    ) -> impl Future<Output = wasmtime::Result<Option<Vec<u8>>>> {
        std::future::ready(
            shared::wired::storage::root_doc_ns(&self.api, self_.rep())
                .map_err(wasmtime::Error::from_anyhow),
        )
    }

    fn registries(
        &mut self,
        self_: Resource<StorageRes>,
    ) -> impl Future<Output = wasmtime::Result<Vec<Vec<u8>>>> {
        std::future::ready(
            shared::wired::storage::registry_namespaces(&self.api, self_.rep())
                .map_err(wasmtime::Error::from_anyhow),
        )
    }

    async fn drop(&mut self, rep: Resource<StorageRes>) -> wasmtime::Result<()> {
        shared::wired::storage::storage_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl HostGetFuture for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<GetFutureRes>,
    ) -> wasmtime::Result<Option<Result<Option<Vec<u8>>, ()>>> {
        shared::wired::storage::get_future_poll(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<GetFutureRes>) -> wasmtime::Result<()> {
        shared::wired::storage::get_future_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl HostListFuture for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<ListFutureRes>,
    ) -> wasmtime::Result<Option<Result<Vec<WitEntry>, ()>>> {
        Ok(
            shared::wired::storage::list_future_poll(&self.api, self_.rep())
                .await
                .map_err(wasmtime::Error::from_anyhow)?
                .map(|res| {
                    res.map(|entries| {
                        entries
                            .into_iter()
                            .map(|e| WitEntry {
                                key:   e.key,
                                value: e.value,
                            })
                            .collect()
                    })
                }),
        )
    }

    async fn drop(&mut self, rep: Resource<ListFutureRes>) -> wasmtime::Result<()> {
        shared::wired::storage::list_future_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl bindings::wired::storage::api::Host for Runtime {
    async fn get_storage(&mut self) -> wasmtime::Result<Result<Resource<Storage>, Error>> {
        if let Err(err) = self.api.require(ApiName::Storage) {
            return Ok(Err(err.into()));
        }
        Ok(shared::wired::storage::get_storage(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(|err| ScriptError::from(err).into()))
    }
}
