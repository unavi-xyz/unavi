//! `wired:perception`: what the local user can perceive of the world around
//! them.

use std::{
    str::FromStr,
    sync::Arc,
};

use bevy::prelude::*;
use bevy_hsd::document::Hsd;
use hsd::id::{
    DocId,
    PrimId,
};
use unavi_policy::{
    permissions::HostApi,
    trust::Trust,
};
use unavi_portal::portal::PortalState;
use xdid::core::did::Did;

use crate::{
    error::ScriptError,
    host::{
        ScriptHost,
        scene::doc_entity,
    },
};

/// A portal in a loaded space, in the calling script's frame.
#[derive(Clone, Copy, Debug)]
pub struct Portal {
    pub doc:         DocId,
    pub prim:        PrimId,
    pub transform:   Transform,
    pub width:       f32,
    pub height:      f32,
    pub destination: Option<DocId>,
    pub linked:      bool,
    pub state:       PortalState,
}

/// What a document declares about itself.
#[derive(Clone, Debug)]
pub struct Meta {
    pub name:        Option<String>,
    pub description: Option<String>,
}

/// Up to 64 portals within `radius` of the camera, nearest first. Empty
/// before the camera exists.
pub fn portals(host: &ScriptHost, radius: f32) -> Result<Vec<Portal>, ScriptError> {
    host.require(HostApi::Perception)?;
    if !radius.is_finite() || radius < 0.0 {
        return Err(ScriptError::invalid(
            "the radius must be finite and not negative",
        ));
    }
    let Some(camera) = host
        .shared
        .agents
        .camera()
        .and_then(|camera| host.shared.transforms.node(&camera.node))
    else {
        return Ok(Vec::new());
    };
    let frame = host.frame();
    Ok(host
        .shared
        .portals
        .near(camera.translation(), radius)
        .into_iter()
        .map(|portal| Portal {
            doc:         portal.doc,
            prim:        portal.prim,
            transform:   frame.transform_to_frame(portal.world),
            width:       portal.width,
            height:      portal.height,
            destination: portal.destination,
            linked:      portal.linked,
            state:       portal.state,
        })
        .collect())
}

/// The meta of `id`, `None` when it is not loaded on this peer.
pub async fn meta(host: &ScriptHost, id: DocId) -> Result<Option<Meta>, ScriptError> {
    host.require(HostApi::Perception)?;
    let state = if id == host.doc {
        Some(Arc::clone(&host.state))
    } else {
        host.world_call(move |world| {
            doc_entity(world, id)
                .and_then(|entity| world.get::<Hsd>(entity))
                .map(Hsd::handle)
        })
        .await?
    };
    state
        .map(|state| {
            let state = state
                .lock()
                .map_err(|_| ScriptError::Internal("document state poisoned".into()))?;
            Ok(Meta {
                name:        state.meta().name.as_ref().map(ToString::to_string),
                description: state.meta().description.as_ref().map(ToString::to_string),
            })
        })
        .transpose()
}

/// How far this peer trusts `user`.
pub fn rung(host: &ScriptHost, user: &str) -> Result<Trust, ScriptError> {
    host.require(HostApi::Perception)?;
    let did = Did::from_str(user).map_err(|_| ScriptError::invalid("not a DID"))?;
    Ok(host.view.trust_of_did(&did))
}
