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
use iroh::EndpointId;
use iroh_docs::NamespaceId;
use unavi_policy::{
    space::Space,
    trust::Trust,
};

use crate::{
    anchor::ActiveSpace,
    connection::PeerLink,
    devtools::inspect::Page,
    state::debug,
    view::SpaceView,
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
    view:   Option<Res<'w, SpaceView>>,
    link:   Option<Res<'w, PeerLink>>,
}

#[derive(std::hash::Hash)]
pub enum PageModel {
    Peer(PeerModel),
    Space(SpaceModel),
    Doc(DocModel),
}

#[derive(std::hash::Hash)]
pub struct PeerModel {
    pub id:        EndpointId,
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
}

#[derive(std::hash::Hash)]
pub struct SpaceModel {
    pub space:  NamespaceId,
    pub joined: bool,
    pub active: bool,
    pub docs:   Vec<SpaceDocRow>,
    pub peers:  Vec<EndpointId>,
}

#[derive(std::hash::Hash)]
pub struct SpaceDocRow {
    pub doc:       NamespaceId,
    pub owner:     Option<EndpointId>,
    pub pins:      usize,
    pub instanced: bool,
}

#[derive(std::hash::Hash)]
pub struct DocModel {
    pub doc:           NamespaceId,
    pub space:         Option<NamespaceId>,
    pub is_space_base: bool,
    pub owner:         Option<EndpointId>,
    pub authority:     Option<EndpointId>,
    pub pinned_by:     Vec<(EndpointId, u64)>,
    pub kv:            Vec<DocKv>,
    pub instanced:     bool,
    pub prims:         Option<usize>,
    pub parent:        Option<NamespaceId>,
    pub subdocs:       Vec<NamespaceId>,
    pub tree:          Option<String>,
}

/// The canonical last-write-wins state of one key, merged across every peer's
/// cells plus the neutral cell.
#[derive(std::hash::Hash)]
pub struct DocKv {
    pub key:    String,
    pub value:  Option<Vec<u8>>,
    pub at:     u64,
    pub writer: EndpointId,
}

impl InspectData<'_, '_> {
    /// `None` until the endpoint and identity exist, since every page is
    /// rendered relative to who the local node is.
    pub fn page_model(&self, page: Page, snap: &debug::DebugSnapshot) -> Option<PageModel> {
        let view = self.view.as_ref()?;
        Some(match page {
            Page::Peer(id) => PageModel::Peer(peer_model(view, self.link.as_deref(), id, snap)),
            Page::Space(space) => PageModel::Space(self.space_model(view, space, snap)),
            Page::Doc(doc) => PageModel::Doc(self.doc_model(view, doc, snap)),
        })
    }
}

pub fn active_space(
    spaces: &Query<(Entity, &Space)>,
    active: Option<Entity>,
) -> Option<NamespaceId> {
    active.and_then(|e| spaces.get(e).ok()).map(|(_, s)| s.0)
}

fn peer_model(
    view: &SpaceView,
    link: Option<&PeerLink>,
    id: EndpointId,
    snap: &debug::DebugSnapshot,
) -> PeerModel {
    let docs = snap
        .peers
        .iter()
        .find(|p| p.peer == id)
        .map(|p| p.docs.as_slice())
        .unwrap_or_default();
    let is_self = view.me() == id;
    PeerModel {
        id,
        is_self,
        connected: link.is_some_and(|l| l.net_stats().iter().any(|s| s.peer == id)),
        did: if is_self {
            Some(view.did())
        } else {
            view.identity().bindings.did_of(id).map(|d| d.to_string())
        },
        trust: view.trust_of(Some(id)),
        pins: docs
            .iter()
            .filter_map(|d| {
                d.pin.map(|at| {
                    (
                        NamespaceId::from(&d.doc.0),
                        NamespaceId::from(&d.space.0),
                        at,
                    )
                })
            })
            .collect(),
        claims: docs
            .iter()
            .filter_map(|d| d.authority.map(|at| (NamespaceId::from(&d.doc.0), at)))
            .collect(),
    }
}

impl InspectData<'_, '_> {
    fn space_model(
        &self,
        view: &SpaceView,
        space: NamespaceId,
        snap: &debug::DebugSnapshot,
    ) -> SpaceModel {
        let space_id = DocId(*space.as_bytes());
        let mut docs = view
            .policy()
            .documents_in(space_id)
            .into_iter()
            .map(|d| NamespaceId::from(&d.0))
            .collect::<Vec<_>>();
        for (doc, doc_space) in snap
            .peers
            .iter()
            .flat_map(|p| p.docs.iter())
            .map(|d| (d.doc, d.space))
            .chain(snap.docs.iter().map(|d| (d.doc, d.space)))
        {
            if doc_space == space_id {
                docs.push(NamespaceId::from(&doc.0));
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
            .filter(|p| p.docs.iter().any(|d| d.space == space_id))
            .map(|p| p.peer)
            .collect::<Vec<_>>();
        peers.sort_unstable();

        SpaceModel {
            space,
            joined: self.spaces.iter().any(|(_, s)| s.0 == space),
            active: active_space(&self.spaces, self.active.0) == Some(space),
            docs: docs
                .into_iter()
                .map(|doc| {
                    let doc_id = DocId(*doc.as_bytes());
                    SpaceDocRow {
                        doc,
                        owner: view.replicas().owner(space_id, doc_id),
                        pins: snap
                            .peers
                            .iter()
                            .filter(|p| p.docs.iter().any(|d| d.doc == doc_id && d.pin.is_some()))
                            .count(),
                        instanced: self
                            .doc_entity(doc)
                            .is_some_and(|(_, instanced, ..)| instanced),
                    }
                })
                .collect(),
            peers,
        }
    }

    fn doc_model(
        &self,
        view: &SpaceView,
        doc: NamespaceId,
        snap: &debug::DebugSnapshot,
    ) -> DocModel {
        let doc_id = DocId(*doc.as_bytes());
        let space = view.space_of(doc_id).map(|s| NamespaceId::from(&s.0));
        let mut pinned_by = snap
            .peers
            .iter()
            .filter_map(|p| {
                p.docs
                    .iter()
                    .find(|d| d.doc == doc_id)
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
            owner: space.and_then(|s| view.replicas().owner(DocId(*s.as_bytes()), doc_id)),
            authority: space.and_then(|s| view.replicas().authority(DocId(*s.as_bytes()), doc_id)),
            pinned_by,
            kv: doc_kv(doc_id, snap),
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

/// `doc`'s cells, ordered by key.
///
/// One cell per key, so the last-write-wins merge already happened when the
/// write landed and there is nothing to resolve here.
fn doc_kv(doc: DocId, snap: &debug::DebugSnapshot) -> Vec<DocKv> {
    let Some(d) = snap.docs.iter().find(|d| d.doc == doc) else {
        return Vec::new();
    };
    let mut cells =
        d.kv.iter()
            .map(|kv| DocKv {
                key:    kv.key.clone(),
                value:  kv.value.clone(),
                at:     kv.at,
                writer: kv.writer,
            })
            .collect::<Vec<_>>();
    cells.sort_unstable_by(|a, b| a.key.cmp(&b.key));
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
    use crate::state::debug::{
        DebugDoc,
        DebugKv,
        DebugSnapshot,
    };

    /// A distinct, valid endpoint id per seed. Arbitrary bytes are not a curve
    /// point, so a key has to be derived rather than written down.
    fn peer(seed: u8) -> EndpointId {
        iroh::SecretKey::from_bytes(&[seed; 32]).public()
    }

    /// The page shows each document's cells, and only that document's.
    #[test]
    fn a_documents_cells_are_listed_by_key() {
        let doc = DocId(*blake3::hash(b"kv-doc").as_bytes());
        let other = DocId(*blake3::hash(b"other-doc").as_bytes());
        let writer = peer(1);

        let cell = |key: &str, at| DebugKv {
            key: key.into(),
            value: Some(b"value".to_vec()),
            at,
            writer,
        };
        let snap = DebugSnapshot {
            peers: Vec::new(),
            docs:  vec![
                DebugDoc {
                    doc,
                    space: doc,
                    kv: vec![cell("b", 2), cell("a", 1)],
                },
                DebugDoc {
                    doc:   other,
                    space: other,
                    kv:    vec![cell("elsewhere", 3)],
                },
            ],
        };

        let cells = doc_kv(doc, &snap);
        assert_eq!(
            cells.iter().map(|c| c.key.as_str()).collect::<Vec<_>>(),
            ["a", "b"],
            "another document's cells must not appear, and keys sort"
        );
        assert_eq!(cells[0].writer, writer);
    }
}
