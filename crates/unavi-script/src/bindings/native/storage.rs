//! `unavi:host/node-storage`.

use wasmtime::component::Resource;

use super::{
    HostCtx,
    convert,
    generated::{
        unavi::host::node_storage::{
            Entry,
            Host,
            HostPendingEntries,
            HostPendingValue,
        },
        wired::core::{
            error::Error,
            ids::DocumentId,
        },
    },
};
use crate::{
    error::ScriptError,
    host::storage::{
        self,
        PendingEntries,
        PendingValue,
    },
};

/// A pending result's error is lowered by hand: it is a value the guest
/// polls, not an error of the call.
fn lower<T>(result: Option<Result<T, ScriptError>>) -> wasmtime::Result<Option<Result<T, Error>>> {
    result
        .map(|result| match result {
            Ok(value) => Ok(Ok(value)),
            Err(err) => convert::error(err).map(Err),
        })
        .transpose()
}

impl Host for HostCtx {
    fn root_document(&mut self) -> Result<Option<DocumentId>, ScriptError> {
        Ok(storage::root_document(&self.host)?.map(convert::wit_doc_id))
    }

    fn registries(&mut self) -> Result<Vec<DocumentId>, ScriptError> {
        Ok(storage::registries(&self.host)?
            .into_iter()
            .map(convert::wit_doc_id)
            .collect())
    }

    async fn get(
        &mut self,
        doc: DocumentId,
        key: String,
    ) -> Result<Resource<PendingValue>, ScriptError> {
        storage::get(&mut self.host, convert::doc_id(doc), key)
            .await
            .map(Resource::new_own)
    }

    async fn list_entries(
        &mut self,
        doc: DocumentId,
        prefix: String,
        limit: u32,
    ) -> Result<Resource<PendingEntries>, ScriptError> {
        storage::list_entries(&mut self.host, convert::doc_id(doc), prefix, limit)
            .await
            .map(Resource::new_own)
    }
}

impl HostPendingValue for HostCtx {
    fn poll(
        &mut self,
        pending: Resource<PendingValue>,
    ) -> wasmtime::Result<Option<Result<Option<Vec<u8>>, Error>>> {
        lower(self.host.pending_values.get(pending.rep())?.poll())
    }

    fn drop(&mut self, pending: Resource<PendingValue>) -> wasmtime::Result<()> {
        self.host.pending_values.remove(pending.rep())?;
        Ok(())
    }
}

impl HostPendingEntries for HostCtx {
    fn poll(
        &mut self,
        pending: Resource<PendingEntries>,
    ) -> wasmtime::Result<Option<Result<Vec<Entry>, Error>>> {
        let entries = self.host.pending_entries.get(pending.rep())?.poll();
        lower(entries.map(|entries| {
            entries.map(|entries| {
                entries
                    .into_iter()
                    .map(|(key, value)| Entry { key, value })
                    .collect()
            })
        }))
    }

    fn drop(&mut self, pending: Resource<PendingEntries>) -> wasmtime::Result<()> {
        self.host.pending_entries.remove(pending.rep())?;
        Ok(())
    }
}
