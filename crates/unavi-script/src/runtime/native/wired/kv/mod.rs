use unavi_space::state::doc::KvError;
use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        wired::kv::KvRes,
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::kv::KvRes;

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-kv",
        with: {
            "wired:kv/types.kv": KvRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::kv::types::{
    HostKv,
    Kv,
    KvError as WitKvError,
};

impl From<KvError> for WitKvError {
    fn from(e: KvError) -> Self {
        match e {
            KvError::QuotaExceeded => Self::QuotaExceeded,
            KvError::KeyTooLong => Self::KeyTooLong,
        }
    }
}

impl bindings::wired::kv::types::Host for Runtime {}

impl HostKv for Runtime {
    async fn get(
        &mut self,
        self_: Resource<KvRes>,
        key: String,
    ) -> wasmtime::Result<Option<Vec<u8>>> {
        shared::wired::kv::kv_get(&self.api, self_.rep(), key)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn set(
        &mut self,
        self_: Resource<KvRes>,
        key: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<(), WitKvError>> {
        shared::wired::kv::kv_set(&self.api, self_.rep(), key, value)
            .await
            .map(|r| r.map_err(WitKvError::from))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn delete(&mut self, self_: Resource<KvRes>, key: String) -> wasmtime::Result<()> {
        shared::wired::kv::kv_delete(&self.api, self_.rep(), key)
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn keys(&mut self, self_: Resource<KvRes>) -> wasmtime::Result<Vec<String>> {
        shared::wired::kv::kv_keys(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<KvRes>) -> wasmtime::Result<()> {
        shared::wired::kv::kv_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl bindings::wired::kv::api::Host for Runtime {
    async fn self_kv(&mut self) -> wasmtime::Result<Resource<Kv>> {
        shared::wired::kv::self_kv(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn get_kv(&mut self, id: Vec<u8>) -> wasmtime::Result<Option<Resource<Kv>>> {
        shared::wired::kv::get_kv(&self.api, id)
            .await
            .map(|opt| opt.map(Resource::new_own))
            .map_err(wasmtime::Error::from_anyhow)
    }
}
