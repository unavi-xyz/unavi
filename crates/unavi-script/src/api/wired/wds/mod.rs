use std::sync::mpsc;

use wasmtime::component::{Resource, ResourceTable};
use wired_records::{BeaconRecord, RecordValue};
use xdid::core::did::Did;

use crate::load::state::RuntimeData;

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-wds",
        with: {
            "wired:wds/types.wds":          super::HostWds,
            "wired:wds/types.query-future": super::HostQueryFuture,
            "wired:wds/types.read-future":  super::HostReadFuture,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::wds::types::{QueryFilter, WdsRecord};

pub struct WiredWdsRt {
    pub actor: Option<wds::actor::Actor>,
    pub table: ResourceTable,
}

pub struct HostWds {
    actor: wds::actor::Actor,
}

pub struct HostQueryFuture {
    rx: mpsc::Receiver<anyhow::Result<Vec<blake3::Hash>>>,
}

pub struct HostReadFuture {
    rx: mpsc::Receiver<anyhow::Result<WdsRecordOut>>,
}

struct WdsRecordOut {
    id: blake3::Hash,
    creator: Did,
    schemas: Vec<blake3::Hash>,
    containers: Vec<(String, Vec<u8>)>,
}

impl bindings::wired::wds::types::Host for RuntimeData {}

impl bindings::wired::wds::context::Host for RuntimeData {
    async fn get_wds(&mut self) -> wasmtime::Result<Resource<HostWds>> {
        let actor = self
            .wired_wds
            .actor
            .clone()
            .ok_or_else(|| wasmtime::Error::msg("wds actor not available"))?;
        Ok(self.wired_wds.table.push(HostWds { actor })?)
    }
}

impl bindings::wired::wds::types::HostWds for RuntimeData {
    async fn query(
        &mut self,
        this: Resource<HostWds>,
        filter: Option<QueryFilter>,
    ) -> wasmtime::Result<Resource<HostQueryFuture>> {
        let actor = self.wired_wds.table.get(&this)?.actor.clone();
        let (tx, rx) = mpsc::channel();

        unavi_wasm_compat::spawn_thread(async move {
            let mut builder = actor.query();

            if let Some(f) = filter {
                if let Some(schemas) = f.schemas {
                    for schema_bytes in schemas {
                        if let Ok(arr) = <[u8; 32]>::try_from(schema_bytes.as_slice()) {
                            builder = builder.schema(blake3::Hash::from_bytes(arr));
                        }
                    }
                }
                if let Some(creator) = f.creator
                    && let Ok(did) = creator.parse()
                {
                    builder = builder.creator(&did);
                }
            }

            let result = builder.send().await;
            let _ = tx.send(result);
        });

        Ok(self.wired_wds.table.push(HostQueryFuture { rx })?)
    }

    async fn read(
        &mut self,
        this: Resource<HostWds>,
        record_id: Vec<u8>,
    ) -> wasmtime::Result<Resource<HostReadFuture>> {
        let actor = self.wired_wds.table.get(&this)?.actor.clone();
        let (tx, rx) = mpsc::channel();

        let id_arr: [u8; 32] = record_id
            .as_slice()
            .try_into()
            .map_err(|_| wasmtime::Error::msg("record id must be 32 bytes"))?;
        let hash = blake3::Hash::from_bytes(id_arr);

        unavi_wasm_compat::spawn_thread(async move {
            let result = read_record(&actor, hash).await;
            let _ = tx.send(result);
        });

        Ok(self.wired_wds.table.push(HostReadFuture { rx })?)
    }

    async fn drop(&mut self, this: Resource<HostWds>) -> wasmtime::Result<()> {
        self.wired_wds.table.delete(this)?;
        Ok(())
    }
}

async fn read_record(
    actor: &wds::actor::Actor,
    hash: blake3::Hash,
) -> anyhow::Result<WdsRecordOut> {
    let doc = actor.read(hash).send().await?;

    let record = wired_schemas::surg::record::Record::load(&doc)?;
    let creator = record.creator.0.clone();
    let id = record.id()?;

    let mut schemas = Vec::new();
    let mut containers = Vec::new();

    for (container, schema_hash) in &record.schemas {
        if container == "acl" || container == "record" {
            continue;
        }
        schemas.push(schema_hash.0);

        let bytes = if container == "beacon" {
            postcard::to_stdvec(&BeaconRecord::load(&doc)?)?
        } else {
            let value = RecordValue::from(doc.get_map(container.as_str()).get_deep_value());
            postcard::to_stdvec(&value)?
        };
        containers.push((container.to_string(), bytes));
    }

    Ok(WdsRecordOut {
        id,
        creator,
        schemas,
        containers,
    })
}

impl bindings::wired::wds::types::HostQueryFuture for RuntimeData {
    async fn poll(
        &mut self,
        this: Resource<HostQueryFuture>,
    ) -> wasmtime::Result<Option<Result<Vec<Vec<u8>>, ()>>> {
        let fut = self.wired_wds.table.get(&this)?;
        match fut.rx.try_recv() {
            Ok(Ok(hashes)) => Ok(Some(Ok(hashes
                .into_iter()
                .map(|h| h.as_bytes().to_vec())
                .collect()))),
            Ok(Err(_)) | Err(mpsc::TryRecvError::Disconnected) => Ok(Some(Err(()))),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
        }
    }

    async fn drop(&mut self, this: Resource<HostQueryFuture>) -> wasmtime::Result<()> {
        self.wired_wds.table.delete(this)?;
        Ok(())
    }
}

impl bindings::wired::wds::types::HostReadFuture for RuntimeData {
    async fn poll(
        &mut self,
        this: Resource<HostReadFuture>,
    ) -> wasmtime::Result<Option<Result<WdsRecord, ()>>> {
        let fut = self.wired_wds.table.get(&this)?;
        match fut.rx.try_recv() {
            Ok(Ok(out)) => {
                let record = WdsRecord {
                    id: out.id.as_bytes().to_vec(),
                    creator: out.creator.to_string(),
                    schemas: out.schemas.iter().map(|h| h.as_bytes().to_vec()).collect(),
                    containers: out.containers,
                };
                Ok(Some(Ok(record)))
            }
            Ok(Err(_)) | Err(mpsc::TryRecvError::Disconnected) => Ok(Some(Err(()))),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
        }
    }

    async fn drop(&mut self, this: Resource<HostReadFuture>) -> wasmtime::Result<()> {
        self.wired_wds.table.delete(this)?;
        Ok(())
    }
}
