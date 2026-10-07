//! `wired:peer`: the local user's identity, and who speaks for each document.

use std::str::FromStr;

use iroh::EndpointId;
use unavi_policy::permissions::HostApi;
use unavi_space::membership::SpaceId;
use xdid::core::did::Did;

use crate::{
    error::ScriptError,
    host::ScriptHost,
};

pub fn self_did(host: &ScriptHost) -> Result<String, ScriptError> {
    host.require(HostApi::Identity)?;
    Ok(host.view.did().to_string())
}

/// The DID of whoever authors `doc`.
pub fn author(host: &ScriptHost, doc: u32) -> Result<Option<String>, ScriptError> {
    host.require(HostApi::Peer)?;
    let id = host.document(doc)?.id;
    Ok(host.view.author(id).map(|did| did.to_string()))
}

/// The DID of the peer holding `doc`, if that peer has proven one.
pub fn holder(host: &ScriptHost, doc: u32) -> Result<Option<String>, ScriptError> {
    host.require(HostApi::Peer)?;
    let Some(peer) = holder_peer(host, doc)? else {
        return Ok(None);
    };
    let did = if peer == host.view.me() {
        Some(host.view.did().clone())
    } else {
        host.view.identity().bindings.did_of(peer)
    };
    Ok(did.map(|did| did.to_string()))
}

fn holder_peer(host: &ScriptHost, doc: u32) -> Result<Option<EndpointId>, ScriptError> {
    let id = host.document(doc)?.id;
    Ok(host
        .view
        .space_of(id)
        .and_then(|space| host.view.replicas().holder(space, id)))
}

pub fn is_author(host: &ScriptHost, doc: u32) -> Result<bool, ScriptError> {
    host.require(HostApi::Peer)?;
    Ok(host.view.is_mine(host.document(doc)?.id))
}

pub fn is_holder(host: &ScriptHost, doc: u32) -> Result<bool, ScriptError> {
    host.require(HostApi::Peer)?;
    Ok(holder_peer(host, doc)? == Some(host.view.me()))
}

fn space(host: &ScriptHost, doc: u32) -> Result<(hsd::id::DocId, SpaceId), ScriptError> {
    let id = host.writable_document(doc)?.id;
    let space = host
        .view
        .space_of(id)
        .ok_or_else(|| ScriptError::invalid("the document is in no space, so it has no holder"))?;
    Ok((id, space))
}

/// Takes hold of a document the script may write. The replica refuses a peer
/// the holder reserved the next hold for.
pub fn take_hold(host: &ScriptHost, doc: u32) -> Result<(), ScriptError> {
    host.require(HostApi::Peer)?;
    let (id, space) = space(host, doc)?;
    host.view.local().take_hold(space, id);
    Ok(())
}

/// Releases this peer's hold on a document the script may write, to `to` if
/// named. Fails with `not-found` when no peer proving `to` is connected.
pub fn release_hold(host: &ScriptHost, doc: u32, to: Option<&str>) -> Result<(), ScriptError> {
    host.require(HostApi::Peer)?;
    let (id, _) = space(host, doc)?;
    let to = to
        .map(|did| {
            let did = Did::from_str(did).map_err(|_| ScriptError::invalid("not a DID"))?;
            host.view
                .identity()
                .bindings
                .peers_of(&did)
                .into_iter()
                .next()
                .ok_or(ScriptError::NotFound)
        })
        .transpose()?;
    host.view.local().release_hold(id, to);
    Ok(())
}
