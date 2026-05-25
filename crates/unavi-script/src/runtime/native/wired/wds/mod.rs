use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        wired::wds::{
            BlobFutureRes,
            QueryFilter,
            QueryFutureRes,
            ReadFutureRes,
            WdsRecord,
            WdsRes,
        },
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::wds::{
        BlobFutureRes,
        QueryFutureRes,
        ReadFutureRes,
        WdsRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-wds",
        with: {
            "wired:wds/types.wds":          WdsRes,
            "wired:wds/types.query-future": QueryFutureRes,
            "wired:wds/types.read-future":  ReadFutureRes,
            "wired:wds/types.blob-future":  BlobFutureRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::wds::types::{
    HostBlobFuture,
    HostQueryFuture,
    HostReadFuture,
    HostWds,
    QueryFilter as WitFilter,
    Wds,
    WdsRecord as WitRecord,
};

impl From<WitFilter> for QueryFilter {
    fn from(f: WitFilter) -> Self {
        Self {
            creator: f.creator,
            schemas: f.schemas,
        }
    }
}

impl From<WdsRecord> for WitRecord {
    fn from(r: WdsRecord) -> Self {
        Self {
            id:         r.id,
            creator:    r.creator,
            schemas:    r.schemas,
            containers: r.containers,
        }
    }
}

impl bindings::wired::wds::types::Host for Runtime {}

impl HostWds for Runtime {
    async fn query(
        &mut self,
        self_: Resource<WdsRes>,
        filter: Option<WitFilter>,
    ) -> wasmtime::Result<Resource<QueryFutureRes>> {
        shared::wired::wds::query(&self.api, self_.rep(), filter.map(Into::into))
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn read(
        &mut self,
        self_: Resource<WdsRes>,
        record_id: Vec<u8>,
    ) -> wasmtime::Result<Resource<ReadFutureRes>> {
        shared::wired::wds::read(&self.api, self_.rep(), record_id)
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

    async fn drop(&mut self, rep: Resource<WdsRes>) -> wasmtime::Result<()> {
        shared::wired::wds::wds_drop(&self.api, rep.rep())
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

impl HostQueryFuture for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<QueryFutureRes>,
    ) -> wasmtime::Result<Option<Result<Vec<Vec<u8>>, ()>>> {
        shared::wired::wds::query_future_poll(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<QueryFutureRes>) -> wasmtime::Result<()> {
        shared::wired::wds::query_future_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl HostReadFuture for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<ReadFutureRes>,
    ) -> wasmtime::Result<Option<Result<WitRecord, ()>>> {
        shared::wired::wds::read_future_poll(&self.api, self_.rep())
            .await
            .map(|opt| opt.map(|r| r.map(Into::into)))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<ReadFutureRes>) -> wasmtime::Result<()> {
        shared::wired::wds::read_future_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl bindings::wired::wds::api::Host for Runtime {
    async fn get_wds(&mut self) -> wasmtime::Result<Resource<Wds>> {
        shared::wired::wds::get_wds(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }
}
