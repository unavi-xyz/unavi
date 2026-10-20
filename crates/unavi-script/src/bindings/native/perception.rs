//! `wired:perception`.

use unavi_policy::trust::Trust;
use unavi_portal::portal::PortalState;

use super::{
    HostCtx,
    convert,
    generated::wired::{
        core::ids::DocumentId,
        perception::perception::{
            Agent,
            DocumentMeta,
            Host,
            Portal,
            PortalState as WitPortalState,
            TrustRung,
            UserProfile,
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
                hue:         convert::wit_color(portal.hue),
            })
            .collect())
    }

    fn agents(&mut self, radius: f32) -> Result<Vec<Agent>, ScriptError> {
        Ok(perception::agents(&self.host, radius)?
            .into_iter()
            .map(|agent| Agent {
                user: agent.user.to_string(),
                body: convert::wit_transform(agent.body),
                head: agent.head.map(convert::wit_transform),
            })
            .collect())
    }

    fn profile(&mut self, user: String) -> Result<Option<UserProfile>, ScriptError> {
        Ok(
            perception::profile(&self.host, &user)?.map(|profile| UserProfile {
                name:        profile.name.map(String::from),
                description: profile.description.map(String::from),
            }),
        )
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
