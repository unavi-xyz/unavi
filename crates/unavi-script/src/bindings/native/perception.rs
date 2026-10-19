//! `wired:perception`.

use unavi_policy::trust::Trust;
use unavi_portal::portal::PortalState;

use super::{
    HostCtx,
    convert,
    generated::wired::{
        core::ids::DocumentId,
        perception::perception::{
            DocumentMeta,
            Host,
            Portal,
            PortalState as WitPortalState,
            TrustRung,
        },
    },
};
use crate::{
    error::ScriptError,
    host::perception,
};

const fn wit_state(state: PortalState) -> WitPortalState {
    match state {
        PortalState::Closed => WitPortalState::Closed,
        PortalState::Loading => WitPortalState::Loading,
        PortalState::Open => WitPortalState::Open,
    }
}

const fn wit_rung(trust: Trust) -> TrustRung {
    match trust {
        Trust::Blocked => TrustRung::Blocked,
        Trust::Anonymous => TrustRung::Anonymous,
        Trust::Guest => TrustRung::Guest,
        Trust::Trusted => TrustRung::Trusted,
        Trust::Myself => TrustRung::Myself,
    }
}

impl Host for HostCtx {
    fn portals(&mut self, radius: f32) -> Result<Vec<Portal>, ScriptError> {
        Ok(perception::portals(&self.host, radius)?
            .into_iter()
            .map(|portal| Portal {
                prim:        convert::wit_prim_ref((portal.doc, portal.prim)),
                transform:   convert::wit_transform(portal.transform),
                width:       portal.width,
                height:      portal.height,
                destination: portal.destination.map(convert::wit_doc_id),
                linked:      portal.linked,
                state:       wit_state(portal.state),
            })
            .collect())
    }

    async fn meta(&mut self, document: DocumentId) -> Result<Option<DocumentMeta>, ScriptError> {
        Ok(perception::meta(&self.host, convert::doc_id(document))
            .await?
            .map(|meta| DocumentMeta {
                name:        meta.name,
                description: meta.description,
            }))
    }

    fn rung(&mut self, user: String) -> Result<TrustRung, ScriptError> {
        perception::rung(&self.host, &user).map(wit_rung)
    }
}
