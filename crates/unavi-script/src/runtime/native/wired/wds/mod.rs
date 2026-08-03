use wasmtime::component::Resource;

use crate::{
    error::ScriptError,
    permissions::ApiName,
    runtime::{
        Runtime,
        shared::{
            self,
            wired::wds::{
                BlobFutureRes,
                GetFutureRes,
                ListFutureRes,
                WdsRes,
            },
        },
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::wds::{
        BlobFutureRes,
        GetFutureRes,
        ListFutureRes,
        WdsRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-wds",
        with: {
            "wired:wds/types.wds":         WdsRes,
            "wired:wds/types.get-future":  GetFutureRes,
            "wired:wds/types.list-future": ListFutureRes,
            "wired:wds/types.blob-future": BlobFutureRes,
            "wired:error/types": crate::runtime::native::wired::error::types,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::wds::types::{
    Entry as WitEntry,
    HostBlobFuture,
    HostGetFuture,
    HostListFuture,
    HostWds,
    Wds,
};

use crate::runtime::native::wired::error::Error;

impl bindings::wired::wds::types::Host for Runtime {}

impl HostWds for Runtime {
    async fn create_doc(&mut self, self_: Resource<WdsRes>) -> wasmtime::Result<Vec<u8>> {
        shared::wired::wds::create_doc(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set(
        &mut self,
        self_: Resource<WdsRes>,
        ns: Vec<u8>,
        key: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<()> {
        shared::wired::wds::set(&self.api, self_.rep(), ns, key, value)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn delete(
        &mut self,
        self_: Resource<WdsRes>,
        ns: Vec<u8>,
        key: String,
    ) -> wasmtime::Result<()> {
        shared::wired::wds::delete(&self.api, self_.rep(), ns, key)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn get(
        &mut self,
        self_: Resource<WdsRes>,
        ns: Vec<u8>,
        key: String,
    ) -> wasmtime::Result<Resource<GetFutureRes>> {
        shared::wired::wds::get(&self.api, self_.rep(), ns, key)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn list(
        &mut self,
        self_: Resource<WdsRes>,
        ns: Vec<u8>,
        prefix: String,
    ) -> wasmtime::Result<Resource<ListFutureRes>> {
        shared::wired::wds::list(&self.api, self_.rep(), ns, prefix)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn get_blob(
        &mut self,
        self_: Resource<WdsRes>,
        blob_id: Vec<u8>,
    ) -> wasmtime::Result<Resource<BlobFutureRes>> {
        shared::wired::wds::get_blob(&self.api, self_.rep(), blob_id)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn root_doc(&mut self, self_: Resource<WdsRes>) -> wasmtime::Result<Option<Vec<u8>>> {
        shared::wired::wds::root_doc_ns(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn registries(&mut self, self_: Resource<WdsRes>) -> wasmtime::Result<Vec<Vec<u8>>> {
        shared::wired::wds::registry_namespaces(&self.api, self_.rep())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<WdsRes>) -> wasmtime::Result<()> {
        shared::wired::wds::wds_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl HostGetFuture for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<GetFutureRes>,
    ) -> wasmtime::Result<Option<Result<Option<Vec<u8>>, ()>>> {
        shared::wired::wds::get_future_poll(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<GetFutureRes>) -> wasmtime::Result<()> {
        shared::wired::wds::get_future_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl HostListFuture for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<ListFutureRes>,
    ) -> wasmtime::Result<Option<Result<Vec<WitEntry>, ()>>> {
        Ok(shared::wired::wds::list_future_poll(&self.api, self_.rep())
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
            }))
    }

    async fn drop(&mut self, rep: Resource<ListFutureRes>) -> wasmtime::Result<()> {
        shared::wired::wds::list_future_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl HostBlobFuture for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<BlobFutureRes>,
    ) -> wasmtime::Result<Option<Result<Vec<u8>, ()>>> {
        shared::wired::wds::blob_future_poll(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<BlobFutureRes>) -> wasmtime::Result<()> {
        shared::wired::wds::blob_future_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl bindings::wired::wds::api::Host for Runtime {
    async fn get_wds(&mut self) -> wasmtime::Result<Result<Resource<Wds>, Error>> {
        if let Err(err) = self.api.require(ApiName::Wds) {
            return Ok(Err(err.into()));
        }
        Ok(shared::wired::wds::get_wds(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(|err| ScriptError::from(err).into()))
    }
}
