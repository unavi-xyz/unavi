//! Writing documents: creating prims, applying edits in a layer, and
//! committing them.

use std::sync::{
    Arc,
    Mutex,
};

use bevy::prelude::*;
use bevy_hsd::{
    document::{
        DocIndex,
        Hsd,
        HsdDocId,
    },
    prim::{
        Prim,
        PrimOf,
    },
};
use hsd::{
    attributes::parent::ParentAttr,
    id::{
        DocId,
        PrimId,
    },
    property::{
        Property as _,
        name::PropName,
        value::Value,
    },
    state::{
        CommitTarget,
        HsdState,
    },
};
use unavi_policy::{
    permissions::HostApi,
    quota::{
        Flow,
        Stock,
    },
};
use unavi_space::replication::message::SessionWrite;

use crate::{
    error::ScriptError,
    host::{
        ScriptHost,
        scene::{
            DocumentRes,
            documents::{
                namespace_of,
                write_entries,
            },
            node,
            property::{
                Property,
                PropertyKey,
            },
        },
    },
};

/// Most edits one `apply` may carry.
pub const MAX_EDITS: usize = 4096;

/// Where a write lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    /// This peer only.
    Local,
    /// Every peer present in the document's space, for the session.
    Shared,
}

#[derive(Clone, Debug)]
pub enum Edit {
    Set(PrimId, Property),
    Clear(PrimId, PropertyKey),
    Remove(PrimId),
}

pub async fn create_prim(
    host: &ScriptHost,
    doc: u32,
    layer: Layer,
    parent: Option<PrimId>,
) -> Result<PrimId, ScriptError> {
    let doc = host.owned_document(doc)?.clone();
    crate::quota::take(&host.quota, Flow::CreatePrim, 1)?;

    if let Some(parent) = parent
        && !doc.read(|state| state.exists(parent))?
    {
        return Err(ScriptError::NotFound);
    }

    match layer {
        Layer::Local => {
            let quota = host.view.document_quota(doc.id);
            quota.charge(Stock::Prims, 1)?;
            doc.write(|state| state.create_prim(parent))
        }
        Layer::Shared => {
            let prim = PrimId::new();
            let parent = parent.map_or(ParentAttr::Root, ParentAttr::Prim);
            let value =
                Some(hsd::property::Payload::encode(&parent).map_err(ScriptError::internal)?);
            share(
                host,
                doc.id,
                vec![SessionWrite {
                    prim,
                    name: ParentAttr::NAME.to_string(),
                    value,
                }],
            )
            .await?;
            Ok(prim)
        }
    }
}

/// Applies every edit in `layer`, all or none.
pub async fn apply(
    host: &ScriptHost,
    doc: u32,
    layer: Layer,
    edits: Vec<Edit>,
) -> Result<(), ScriptError> {
    let doc = host.owned_document(doc)?.clone();
    if edits.len() > MAX_EDITS {
        return Err(ScriptError::invalid("at most 4096 edits per apply"));
    }
    let mut uploads = 0;
    for edit in &edits {
        if let Edit::Set(_, property) = edit {
            property.check()?;
            uploads += u32::from(property.is_upload());
        }
    }
    if uploads > 0 {
        crate::quota::take(&host.quota, Flow::BlobUpload, uploads)?;
    }

    match layer {
        Layer::Local => {
            let removed = doc.write(|state| apply_local(state, &edits))??;
            host.view
                .document_quota(doc.id)
                .release(Stock::Prims, removed);
        }
        Layer::Shared => {
            let writes = edits
                .iter()
                .map(shared_write)
                .collect::<Result<Vec<_>, _>>()?;
            share(host, doc.id, writes).await?;
        }
    }

    // A script reads back the transform it just wrote without waiting a
    // frame for the next snapshot.
    for edit in &edits {
        if let Edit::Set(prim, Property::Transform(xform)) = edit {
            host.shared.transforms.set_local(
                &node(doc.id, *prim),
                Transform {
                    translation: Vec3::from_array(xform.translation),
                    rotation:    Quat::from_array(xform.rotation),
                    scale:       Vec3::from_array(xform.scale),
                },
            );
        }
    }
    Ok(())
}

/// Checks every edit against `state`, then applies them all, answering how
/// many prims left the scene.
fn apply_local(state: &mut HsdState, edits: &[Edit]) -> Result<u64, ScriptError> {
    for edit in edits {
        let prim = match edit {
            Edit::Set(prim, Property::Parent(Some(parent))) => {
                if !state.exists(*parent) {
                    return Err(ScriptError::NotFound);
                }
                if is_beneath(state, *parent, *prim) {
                    return Err(ScriptError::invalid(
                        "a prim cannot be moved beneath itself",
                    ));
                }
                prim
            }
            Edit::Clear(_, PropertyKey::Parent) => {
                return Err(ScriptError::invalid(
                    "`parent` cannot be cleared; remove the prim",
                ));
            }
            Edit::Set(prim, _) | Edit::Clear(prim, _) | Edit::Remove(prim) => prim,
        };
        if !state.exists(*prim) {
            return Err(ScriptError::NotFound);
        }
    }

    let before = state.prims().count();
    for edit in edits {
        match edit {
            Edit::Set(prim, Property::Parent(parent)) => state
                .set_parent(*prim, parent.map_or(ParentAttr::Root, ParentAttr::Prim))
                .map_err(ScriptError::internal)?,
            Edit::Set(prim, Property::Relation(relation, target)) => {
                let name = PropertyKey::Relation(relation.clone()).name()?;
                state
                    .set_relationship(*prim, &name, *target)
                    .map_err(ScriptError::internal)?;
            }
            Edit::Set(prim, property) => {
                let name = property.key().name()?;
                let payload = property.payload()?.unwrap_or_default();
                state
                    .set_property(*prim, &name, Value::Attribute(payload.into()))
                    .map_err(ScriptError::internal)?;
            }
            Edit::Clear(prim, key) => state.remove_property(*prim, &key.name()?),
            Edit::Remove(prim) => state.remove_prim(*prim),
        }
    }
    Ok(before.saturating_sub(state.prims().count()) as u64)
}

/// Writes raw fields of one prim in `layer`, all or none. `None` clears a
/// field locally, and blocks it in the shared layer.
pub async fn write_fields(
    host: &ScriptHost,
    doc: &DocumentRes,
    layer: Layer,
    prim: PrimId,
    fields: Vec<(PropName, Option<Vec<u8>>)>,
) -> Result<(), ScriptError> {
    match layer {
        Layer::Local => doc.write(|state| {
            if !state.exists(prim) {
                return Err(ScriptError::NotFound);
            }
            for (name, value) in fields {
                match value {
                    Some(bytes) => state
                        .set_property(prim, &name, Value::Attribute(bytes.into()))
                        .map_err(ScriptError::internal)?,
                    None => state.remove_property(prim, &name),
                }
            }
            Ok(())
        })?,
        Layer::Shared => {
            let writes = fields
                .into_iter()
                .map(|(name, value)| SessionWrite {
                    prim,
                    name: name.to_string(),
                    value,
                })
                .collect();
            share(host, doc.id, writes).await
        }
    }
}

/// Whether `prim` lies at or beneath `ancestor`.
fn is_beneath(state: &HsdState, prim: PrimId, ancestor: PrimId) -> bool {
    let mut at = Some(prim);
    while let Some(current) = at {
        if current == ancestor {
            return true;
        }
        at = state.parent(current);
    }
    false
}

/// An edit as a session write. The shared layer holds bytes alone, so a
/// relation, which is a link, cannot be shared.
fn shared_write(edit: &Edit) -> Result<SessionWrite, ScriptError> {
    let (prim, name, value) = match edit {
        Edit::Set(_, Property::Relation(..)) => {
            return Err(ScriptError::invalid("a relation cannot be shared"));
        }
        Edit::Set(prim, property) => (*prim, property.key().name()?, property.payload()?),
        Edit::Clear(prim, key) => (*prim, key.name()?, None),
        Edit::Remove(prim) => (*prim, ParentAttr::NAME, None),
    };
    Ok(SessionWrite {
        prim,
        name: name.to_string(),
        value,
    })
}

/// States `writes` on `doc` for the session, as one batch.
async fn share(
    host: &ScriptHost,
    doc: DocId,
    writes: Vec<SessionWrite>,
) -> Result<(), ScriptError> {
    let space = host.view.space_of(doc).ok_or_else(|| {
        ScriptError::invalid("the document is in no space, so it has no shared layer")
    })?;
    Ok(host.view.local().set_session(space, doc, writes).await?)
}

/// Where a commit against a document lands.
enum Landing {
    /// This node holds the document's own write key.
    Document,
    /// It holds the key of the document referencing it, so the commit is an
    /// override there.
    Override {
        site:      PrimId,
        reference: DocId,
        state:     Arc<Mutex<HsdState>>,
    },
}

/// Makes the live value of each key durable.
///
/// Does nothing unless this peer holds the script's document: every peer runs
/// the script, and only the holder's copy speaks for it. A script outside any
/// space runs on this peer alone.
pub async fn commit(
    host: &ScriptHost,
    doc: u32,
    keys: Vec<(PrimId, PropertyKey)>,
) -> Result<(), ScriptError> {
    host.require(HostApi::Commit)?;
    let doc = host.owned_document(doc)?.clone();

    if let Some(space) = host.view.space_of(host.doc)
        && !host
            .view
            .replicas()
            .is_holder(space, host.doc, host.view.me())
    {
        return Ok(());
    }

    let props = keys
        .iter()
        .map(|(prim, key)| Ok((*prim, key.name()?)))
        .collect::<Result<Vec<(PrimId, PropName)>, ScriptError>>()?;

    let landing = landing(host, doc.id).await?;
    let target = match &landing {
        Landing::Document => CommitTarget::Document,
        Landing::Override { site, .. } => CommitTarget::Override { site: *site },
    };
    let entries = doc.write(|state| state.commit(target, &props))?;

    match landing {
        Landing::Document => write_entries(host, namespace_of(host, doc.id).await?, entries).await,
        Landing::Override {
            reference, state, ..
        } => {
            let referencing = DocumentRes {
                id: reference,
                state,
            };
            referencing.write(|state| {
                entries
                    .iter()
                    .try_for_each(|entry| state.project(entry))
                    .map_err(ScriptError::internal)
            })??;
            write_entries(host, namespace_of(host, reference).await?, entries).await
        }
    }
}

/// Its own key, else the key of the document referencing it; anything else
/// is `forbidden`. One hop by construction: an override names a prim of a
/// document the overriding one references directly.
async fn landing(host: &ScriptHost, doc: DocId) -> Result<Landing, ScriptError> {
    if holds_write_key(host, doc).await? {
        return Ok(Landing::Document);
    }
    let site = host
        .world_call(move |world| reference_site(world, doc))
        .await?
        .ok_or(ScriptError::Forbidden)?;
    let (site, reference, state) = site;
    if !holds_write_key(host, reference).await? {
        return Err(ScriptError::Forbidden);
    }
    Ok(Landing::Override {
        site,
        reference,
        state,
    })
}

async fn holds_write_key(host: &ScriptHost, doc: DocId) -> Result<bool, ScriptError> {
    let ns = namespace_of(host, doc).await?;
    let store = host
        .world_call(|world| {
            world
                .get_resource::<bevy_iroh::store::DataStore>()
                .map(|s| s.0.clone())
        })
        .await?;
    let Some(store) = store else {
        return Ok(false);
    };
    Ok(store
        .write_key(ns)
        .await
        .map_err(ScriptError::internal)?
        .is_some())
}

/// The prim a reference in the scene hangs from, and the document holding it.
/// A reference is spawned as a child of the prim naming it.
fn reference_site(world: &World, doc: DocId) -> Option<(PrimId, DocId, Arc<Mutex<HsdState>>)> {
    let entity = world.get_resource::<DocIndex>()?.get(doc)?;
    let prim_ent = world.get::<ChildOf>(entity)?.0;
    let site = world.get::<Prim>(prim_ent)?.0;
    let host = world.get::<PrimOf>(prim_ent)?.0;
    Some((
        site,
        world.get::<HsdDocId>(host)?.0,
        Arc::clone(&world.get::<Hsd>(host)?.0),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(n: u8) -> DocId {
        DocId([n; 32])
    }

    /// A document, a prim of it, and the reference that prim adds to the
    /// scene, spawned the way references are opened.
    fn scene_reference(world: &mut World) -> (PrimId, DocId, DocId) {
        world.init_resource::<DocIndex>();
        let host = doc(1);
        let host_ent = world
            .spawn((Hsd::new(HsdState::new()), HsdDocId(host)))
            .id();
        let site = PrimId::new();
        let prim_ent = world.spawn((Prim(site), PrimOf(host_ent))).id();
        let child = DocId::site(host, site);
        world.spawn((
            Hsd::new(HsdState::new()),
            HsdDocId(child),
            ChildOf(prim_ent),
        ));
        (site, host, child)
    }

    #[test]
    fn a_reference_answers_the_prim_and_document_holding_it() {
        let mut world = World::new();
        let (site, host, child) = scene_reference(&mut world);

        let (found_site, found_host, _) =
            reference_site(&world, child).expect("the site is reachable from the child");

        assert_eq!(found_site, site, "which key an override takes");
        assert_eq!(found_host, host, "and whose document it belongs in");
    }

    #[test]
    fn a_document_nothing_references_has_no_site() {
        let mut world = World::new();
        scene_reference(&mut world);
        let placed = doc(2);
        world.spawn((Hsd::new(HsdState::new()), HsdDocId(placed)));

        assert!(reference_site(&world, placed).is_none());
    }

    #[test]
    fn a_batch_with_one_bad_edit_changes_nothing() {
        let mut state = HsdState::new();
        let prim = state.create_prim(None);
        let edits = [
            Edit::Set(prim, Property::Name("kept out".into())),
            Edit::Remove(PrimId::new()),
        ];
        assert_eq!(apply_local(&mut state, &edits), Err(ScriptError::NotFound));
        assert_eq!(
            PropertyKey::Name.read(&state, prim),
            None,
            "the valid edit before the bad one did not land"
        );
    }

    #[test]
    fn a_prim_cannot_move_beneath_itself() {
        let mut state = HsdState::new();
        let parent = state.create_prim(None);
        let child = state.create_prim(Some(parent));
        assert!(
            apply_local(
                &mut state,
                &[Edit::Set(parent, Property::Parent(Some(child)))]
            )
            .is_err()
        );
    }
}
