use std::fmt::Write;

use bevy::{
    ecs::system::SystemParam,
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdNamespace,
    HsdPrimIndex,
};
use hsd::{
    attributes::{
        Attribute,
        name::NameAttr,
    },
    id::{
        DocId,
        PrimId,
    },
    state::SceneState,
};
use iroh_docs::NamespaceId;
use unavi_policy::{
    check,
    identity,
    registry,
    space::Space,
    trust::Trust,
};

use crate::{
    anchor::ActiveSpace,
    devtools::{
        conn,
        inspect::Page,
    },
    peer::{
        self_did,
        self_peer_id,
    },
    state::replicas::{
        self,
        debug,
    },
};

const HSD_TREE_MAX_PRIMS: usize = 200;

#[derive(SystemParam)]
pub struct InspectData<'w, 's> {
    spaces: Query<'w, 's, (Entity, &'static Space)>,
    active: Res<'w, ActiveSpace>,
    docs: Query<
        'w,
        's,
        (
            Entity,
            &'static HsdNamespace,
            Has<Hsd>,
            Option<&'static HsdPrimIndex>,
            Option<&'static ChildOf>,
        ),
    >,
    hsds:   Query<'w, 's, &'static Hsd>,
    prims:  Query<'w, 's, &'static HsdChild>,
}

#[derive(std::hash::Hash)]
pub enum PageModel {
    Peer(PeerModel),
    Space(SpaceModel),
    Doc(DocModel),
}

#[derive(std::hash::Hash)]
pub struct PeerModel {
    pub id:        [u8; 32],
    pub is_self:   bool,
    pub connected: bool,
    /// The DID this peer proved over its own connection, absent while it has
    /// proved none.
    pub did:       Option<String>,
    /// The rung the peer sits at, which is what every cross-owner write is
    /// judged against.
    pub trust:     Trust,
    /// Pinned docs as (doc, space, pinned-at).
    pub pins:      Vec<(NamespaceId, NamespaceId, u64)>,
    pub claims:    Vec<(NamespaceId, u64)>,
    pub kv:        Vec<PeerKvRow>,
}

#[derive(std::hash::Hash)]
pub struct PeerKvRow {
    pub doc:   NamespaceId,
    pub key:   String,
    pub value: Option<Vec<u8>>,
    pub at:    u64,
}

#[derive(std::hash::Hash)]
pub struct SpaceModel {
    pub space:  NamespaceId,
    pub joined: bool,
    pub active: bool,
    pub docs:   Vec<SpaceDocRow>,
    pub peers:  Vec<[u8; 32]>,
}

#[derive(std::hash::Hash)]
pub struct SpaceDocRow {
    pub doc:       NamespaceId,
    pub owner:     Option<[u8; 32]>,
    pub pins:      usize,
    pub instanced: bool,
}

#[derive(std::hash::Hash)]
pub struct DocModel {
    pub doc:           NamespaceId,
    pub space:         Option<NamespaceId>,
    pub is_space_base: bool,
    pub owner:         Option<[u8; 32]>,
    pub authority:     Option<[u8; 32]>,
    pub pinned_by:     Vec<([u8; 32], u64)>,
    pub kv:            Vec<MergedKv>,
    pub instanced:     bool,
    pub prims:         Option<usize>,
    pub parent:        Option<NamespaceId>,
    pub subdocs:       Vec<NamespaceId>,
    pub tree:          Option<String>,
}

/// The canonical last-write-wins state of one key, merged across every peer's
/// cells plus the neutral cell.
#[derive(std::hash::Hash)]
pub struct MergedKv {
    pub key:     String,
    pub value:   Option<Vec<u8>>,
    pub at:      u64,
    pub writer:  [u8; 32],
    pub neutral: bool,
}

impl InspectData<'_, '_> {
    pub fn page_model(&self, page: Page, snap: &debug::DebugSnapshot) -> PageModel {
        match page {
            Page::Peer(id) => PageModel::Peer(peer_model(id, snap)),
            Page::Space(space) => PageModel::Space(self.space_model(space, snap)),
            Page::Doc(doc) => PageModel::Doc(self.doc_model(doc, snap)),
        }
    }
}

pub fn active_space(
    spaces: &Query<(Entity, &Space)>,
    active: Option<Entity>,
) -> Option<NamespaceId> {
    active.and_then(|e| spaces.get(e).ok()).map(|(_, s)| s.0)
}

fn peer_model(id: [u8; 32], snap: &debug::DebugSnapshot) -> PeerModel {
    let docs = snap
        .peers
        .iter()
        .find(|p| p.peer == id)
        .map(|p| p.docs.as_slice())
        .unwrap_or_default();
    let is_self = self_peer_id() == Some(id);
    PeerModel {
        id,
        is_self,
        connected: conn::snapshot().iter().any(|s| s.peer == id),
        did: if is_self {
            self_did()
        } else {
            identity::did_of(id).map(|d| d.to_string())
        },
        trust: check::trust_of(Some(id)),
        pins: docs
            .iter()
            .filter_map(|d| d.pin.map(|at| (d.doc, d.space, at)))
            .collect(),
        claims: docs
            .iter()
            .filter_map(|d| d.authority.map(|at| (d.doc, at)))
            .collect(),
        kv: docs
            .iter()
            .flat_map(|d| {
                d.kv.iter().map(|kv| PeerKvRow {
                    doc:   d.doc,
                    key:   kv.key.clone(),
                    value: kv.value.clone(),
                    at:    kv.at,
                })
            })
            .collect(),
    }
}

impl InspectData<'_, '_> {
    fn space_model(&self, space: NamespaceId, snap: &debug::DebugSnapshot) -> SpaceModel {
        let mut docs = registry::documents_in(DocId(*space.as_bytes()))
            .into_iter()
            .map(|d| NamespaceId::from(&d.0))
            .collect::<Vec<_>>();
        for d in snap
            .peers
            .iter()
            .flat_map(|p| p.docs.iter())
            .chain(snap.neutral.iter())
        {
            if d.space == space {
                docs.push(d.doc);
            }
        }
        docs.sort_unstable_by_key(|d| *d.as_bytes());
        docs.dedup();
        if let Some(pos) = docs.iter().position(|d| *d == space) {
            let base = docs.remove(pos);
            docs.insert(0, base);
        }

        let mut peers = snap
            .peers
            .iter()
            .filter(|p| p.docs.iter().any(|d| d.space == space))
            .map(|p| p.peer)
            .collect::<Vec<_>>();
        peers.sort_unstable();

        SpaceModel {
            space,
            joined: self.spaces.iter().any(|(_, s)| s.0 == space),
            active: active_space(&self.spaces, self.active.0) == Some(space),
            docs: docs
                .into_iter()
                .map(|doc| SpaceDocRow {
                    doc,
                    owner: replicas::owner(space, doc),
                    pins: snap
                        .peers
                        .iter()
                        .filter(|p| p.docs.iter().any(|d| d.doc == doc && d.pin.is_some()))
                        .count(),
                    instanced: self
                        .doc_entity(doc)
                        .is_some_and(|(_, instanced, ..)| instanced),
                })
                .collect(),
            peers,
        }
    }

    fn doc_model(&self, doc: NamespaceId, snap: &debug::DebugSnapshot) -> DocModel {
        let space = check::space_of(DocId(*doc.as_bytes())).map(|s| NamespaceId::from(&s.0));
        let mut pinned_by = snap
            .peers
            .iter()
            .filter_map(|p| {
                p.docs
                    .iter()
                    .find(|d| d.doc == doc)
                    .and_then(|d| d.pin)
                    .map(|at| (p.peer, at))
            })
            .collect::<Vec<_>>();
        pinned_by.sort_unstable();

        let entity = self.doc_entity(doc);
        let mut subdocs = self
            .docs
            .iter()
            .filter(|(e, ..)| self.parent_doc(*e) == Some(doc))
            .map(|(_, record, ..)| record.0)
            .collect::<Vec<_>>();
        subdocs.sort_unstable_by_key(|d| *d.as_bytes());

        DocModel {
            doc,
            space,
            is_space_base: space == Some(doc),
            owner: space.and_then(|s| replicas::owner(s, doc)),
            authority: space.and_then(|s| replicas::authority(s, doc)),
            pinned_by,
            kv: merged_kv(doc, snap),
            instanced: entity.is_some_and(|(_, instanced, ..)| instanced),
            prims: entity.and_then(|(.., prims, _)| prims),
            parent: entity.and_then(|(e, ..)| self.parent_doc(e)),
            subdocs,
            tree: entity.and_then(|(e, ..)| {
                self.hsds
                    .get(e)
                    .ok()
                    .and_then(|hsd| hsd.0.lock().ok().map(|state| hsd_tree_text(&state)))
            }),
        }
    }

    fn doc_entity(
        &self,
        doc: NamespaceId,
    ) -> Option<(Entity, bool, Option<usize>, Option<Entity>)> {
        self.docs
            .iter()
            .find(|(_, record, ..)| record.0 == doc)
            .map(|(e, _, instanced, prims, parent)| {
                (
                    e,
                    instanced,
                    prims.map(|p| p.0.len()),
                    parent.map(ChildOf::parent),
                )
            })
    }

    /// Resolves the document containing `entity`: its Bevy parent is a prim,
    /// whose [`HsdChild`] points at the owning document.
    fn parent_doc(&self, entity: Entity) -> Option<NamespaceId> {
        let (.., parent) = self.docs.get(entity).ok()?;
        let prim = parent.map(ChildOf::parent)?;
        let owner = self.prims.get(prim).ok()?.0;
        self.docs.get(owner).ok().map(|(_, record, ..)| record.0)
    }
}

fn merged_kv(doc: NamespaceId, snap: &debug::DebugSnapshot) -> Vec<MergedKv> {
    let mut cells = Vec::new();
    for p in &snap.peers {
        if let Some(d) = p.docs.iter().find(|d| d.doc == doc) {
            for kv in &d.kv {
                cells.push(MergedKv {
                    key:     kv.key.clone(),
                    value:   kv.value.clone(),
                    at:      kv.at,
                    writer:  p.peer,
                    neutral: false,
                });
            }
        }
    }
    if let Some(d) = snap.neutral.iter().find(|d| d.doc == doc) {
        for kv in &d.kv {
            cells.push(MergedKv {
                key:     kv.key.clone(),
                value:   kv.value.clone(),
                at:      kv.at,
                writer:  kv.writer.unwrap_or_default(),
                neutral: true,
            });
        }
    }
    // Last write wins per key, tying by writer id, mirroring the resolver.
    cells.sort_unstable_by(|a, b| {
        a.key
            .cmp(&b.key)
            .then_with(|| b.at.cmp(&a.at))
            .then_with(|| b.writer.cmp(&a.writer))
    });
    cells.dedup_by(|next, winner| next.key == winner.key);
    cells
}

fn hsd_tree_text(state: &SceneState) -> String {
    let mut out = String::new();
    let mut budget = HSD_TREE_MAX_PRIMS;
    for root in state.roots() {
        walk_prim(state, root, 0, &mut out, &mut budget);
    }
    if budget == 0 {
        let _ = write!(out, "…");
    }
    if out.is_empty() {
        out.push_str("(empty)");
    }
    out
}

fn walk_prim(state: &SceneState, id: PrimId, depth: usize, out: &mut String, budget: &mut usize) {
    if *budget == 0 {
        return;
    }
    *budget -= 1;
    let (name, attrs) = prim_summary(state, id);
    let _ = writeln!(out, "{:indent$}{name} [{attrs}]", "", indent = depth * 2);
    for child in state.children(id) {
        walk_prim(state, child, depth + 1, out, budget);
    }
}

fn prim_summary(state: &SceneState, id: PrimId) -> (String, String) {
    let mut name = "prim".to_string();
    let mut keys = Vec::new();
    if let Some(prim) = state.get(id) {
        for (key, _) in prim.properties() {
            if key == NameAttr::KEY {
                if let Some(Ok(n)) = state.attribute::<NameAttr>(id) {
                    name = format!("{:?}", n.0);
                }
                continue;
            }
            keys.push(key.to_string());
        }
        keys.extend(prim.slots().map(|(slot, _)| slot.to_string()));
        keys.sort_unstable();
    }
    (name, keys.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::replicas::debug::{
        DebugDoc,
        DebugKv,
        DebugPeer,
        DebugSnapshot,
    };

    fn doc_with_kv(doc: NamespaceId, kv: Vec<DebugKv>) -> DebugDoc {
        DebugDoc {
            doc,
            space: doc,
            pin: None,
            authority: None,
            kv,
        }
    }

    #[test]
    fn merged_kv_latest_write_wins_across_peers_and_neutral() {
        let doc = NamespaceId::from(blake3::hash(b"merged-kv-doc").as_bytes());
        let early = [1u8; 32];
        let late = [2u8; 32];
        let snap = DebugSnapshot {
            peers:   vec![
                DebugPeer {
                    peer: early,
                    docs: vec![doc_with_kv(
                        doc,
                        vec![DebugKv {
                            key:    "k".into(),
                            value:  Some(b"old".to_vec()),
                            at:     1,
                            writer: None,
                        }],
                    )],
                },
                DebugPeer {
                    peer: late,
                    docs: vec![doc_with_kv(
                        doc,
                        vec![DebugKv {
                            key:    "k".into(),
                            value:  Some(b"new".to_vec()),
                            at:     2,
                            writer: None,
                        }],
                    )],
                },
            ],
            neutral: vec![doc_with_kv(
                doc,
                vec![DebugKv {
                    key:    "n".into(),
                    value:  Some(b"space".to_vec()),
                    at:     3,
                    writer: Some(early),
                }],
            )],
        };

        let merged = merged_kv(doc, &snap);
        assert_eq!(merged.len(), 2);
        let k = merged.iter().find(|c| c.key == "k").expect("k merged");
        assert_eq!(k.value.as_deref(), Some(&b"new"[..]));
        assert_eq!(k.writer, late);
        assert!(!k.neutral);
        let n = merged.iter().find(|c| c.key == "n").expect("n merged");
        assert_eq!(n.writer, early);
        assert!(n.neutral);
    }
}
