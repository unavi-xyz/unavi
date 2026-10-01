//! `wired:scene`: documents, and reading the prims they hold.

use std::sync::{
    Arc,
    Mutex,
};

use bevy::prelude::*;
use bevy_hsd::document::{
    DocIndex,
    Hsd,
};
use hsd::{
    attributes::name::NameAttr,
    id::{
        DocId,
        PrimId,
    },
    state::HsdState,
};
use unavi_policy::permissions::HostApi;

use crate::{
    error::ScriptError,
    host::{
        ScriptHost,
        scene::property::{
            Property,
            PropertyKey,
        },
        shared_state::transforms::AbsoluteNodeId,
    },
};

pub mod documents;
pub mod edit;
pub mod place;
pub mod property;

/// A guest's handle to a document. Holding one is the capability to read it.
#[derive(Clone)]
pub struct DocumentRes {
    pub id:    DocId,
    pub state: Arc<Mutex<HsdState>>,
}

impl DocumentRes {
    pub fn read<T>(&self, f: impl FnOnce(&HsdState) -> T) -> Result<T, ScriptError> {
        let state = self
            .state
            .lock()
            .map_err(|_| ScriptError::Internal("document state poisoned".into()))?;
        Ok(f(&state))
    }

    pub fn write<T>(&self, f: impl FnOnce(&mut HsdState) -> T) -> Result<T, ScriptError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| ScriptError::Internal("document state poisoned".into()))?;
        Ok(f(&mut state))
    }
}

fn insert(host: &mut ScriptHost, doc: DocumentRes) -> Result<u32, ScriptError> {
    Ok(host.documents.insert(doc, &host.quota)?)
}

pub fn script_document(host: &mut ScriptHost) -> Result<u32, ScriptError> {
    host.require(HostApi::Scene)?;
    let doc = DocumentRes {
        id:    host.doc,
        state: Arc::clone(&host.state),
    };
    insert(host, doc)
}

#[must_use]
pub const fn script_prim(host: &ScriptHost) -> PrimId {
    host.prim
}

/// A document loaded on this peer, `None` when it is not.
pub async fn open_document(host: &mut ScriptHost, id: DocId) -> Result<Option<u32>, ScriptError> {
    host.require(HostApi::Scene)?;
    let state = if id == host.doc {
        Some(Arc::clone(&host.state))
    } else {
        host.world_call(move |world| {
            let entity = world.get_resource::<DocIndex>()?.get(id)?;
            world.get::<Hsd>(entity).map(|doc| Arc::clone(&doc.0))
        })
        .await?
    };
    state
        .map(|state| insert(host, DocumentRes { id, state }))
        .transpose()
}

pub fn drop_document(host: &mut ScriptHost, doc: u32) -> Result<(), ScriptError> {
    host.documents.remove(doc).map(drop)
}

pub fn id(host: &ScriptHost, doc: u32) -> Result<DocId, ScriptError> {
    Ok(host.document(doc)?.id)
}

pub fn owned(host: &ScriptHost, doc: u32) -> Result<bool, ScriptError> {
    Ok(host.owns(host.document(doc)?.id))
}

pub fn contains(host: &ScriptHost, doc: u32, prim: PrimId) -> Result<bool, ScriptError> {
    host.document(doc)?.read(|state| state.is_in_scene(prim))
}

pub fn roots(host: &ScriptHost, doc: u32) -> Result<Vec<PrimId>, ScriptError> {
    host.document(doc)?.read(HsdState::roots)
}

pub fn prims(host: &ScriptHost, doc: u32) -> Result<Vec<PrimId>, ScriptError> {
    host.document(doc)?.read(|state| state.prims().collect())
}

pub fn children(host: &ScriptHost, doc: u32, prim: PrimId) -> Result<Vec<PrimId>, ScriptError> {
    host.document(doc)?.read(|state| state.children(prim))
}

pub fn find_by_name(host: &ScriptHost, doc: u32, name: &str) -> Result<Vec<PrimId>, ScriptError> {
    host.document(doc)?.read(|state| {
        state
            .prims()
            .filter(|prim| {
                state
                    .attribute::<NameAttr>(*prim)
                    .and_then(Result::ok)
                    .is_some_and(|found| found.0 == name)
            })
            .collect()
    })
}

pub fn get(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
    key: &PropertyKey,
) -> Result<Option<Property>, ScriptError> {
    host.document(doc)?.read(|state| key.read(state, prim))
}

pub fn get_many(
    host: &ScriptHost,
    doc: u32,
    keys: &[(PrimId, PropertyKey)],
) -> Result<Vec<Option<Property>>, ScriptError> {
    host.document(doc)?.read(|state| {
        keys.iter()
            .map(|(prim, key)| key.read(state, *prim))
            .collect()
    })
}

pub fn keys(host: &ScriptHost, doc: u32, prim: PrimId) -> Result<Vec<PropertyKey>, ScriptError> {
    host.document(doc)?
        .read(|state| PropertyKey::all(state, prim))
}

/// Where `prim` is in the world this frame, composed from the transform
/// snapshot so physics and animation show.
pub fn world_transform(
    host: &ScriptHost,
    doc: u32,
    prim: PrimId,
) -> Result<Transform, ScriptError> {
    let doc = host.document(doc)?;
    let state = Arc::clone(&doc.state);
    if !doc.read(|state| state.is_in_scene(prim))? {
        return Err(ScriptError::NotFound);
    }
    let affine = host.shared.transforms.world_of(doc.id, prim, |id| {
        state.lock().ok().and_then(|state| state.parent(id))
    });
    Ok(Transform::from_matrix(affine.into()))
}

/// The transform from `doc`'s root to `other`'s, `None` when either is not
/// placed.
pub fn offset_to(
    host: &ScriptHost,
    doc: u32,
    other: u32,
) -> Result<Option<Transform>, ScriptError> {
    let transforms = &host.shared.transforms;
    let (Some(from), Some(to)) = (
        transforms.doc_root(&host.document(doc)?.id),
        transforms.doc_root(&host.document(other)?.id),
    ) else {
        return Ok(None);
    };
    let relative = from.affine().inverse() * to.affine();
    Ok(Some(Transform::from_matrix(relative.into())))
}

/// The id of `prim` in the transform snapshot.
#[must_use]
pub const fn node(doc: DocId, prim: PrimId) -> AbsoluteNodeId {
    AbsoluteNodeId { doc, node: prim }
}
