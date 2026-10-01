//! Rendering an inspector page model into UI.

use bevy::{
    ecs::relationship::RelatedSpawnerCommands,
    prelude::*,
};
use iroh_docs::NamespaceId;
use unavi_policy::trust::Trust;

use crate::inspect::{
    BackButton,
    ExpandButton,
    Expanded,
    Page,
    TrustButton,
    model::{
        DocModel,
        PageModel,
        PeerModel,
        SpaceModel,
    },
    widgets,
};

pub fn build(
    b: &mut RelatedSpawnerCommands<ChildOf>,
    model: &PageModel,
    expanded: &Expanded,
    can_back: bool,
) {
    header(b, model, can_back);
    match model {
        PageModel::Peer(m) => peer_page(b, m),
        PageModel::Space(m) => space_page(b, m),
        PageModel::Doc(m) => doc_page(b, m, expanded),
    }
}

fn header(b: &mut RelatedSpawnerCommands<ChildOf>, model: &PageModel, can_back: bool) {
    b.spawn(widgets::row_node()).with_children(|r| {
        if can_back {
            widgets::small_button(r, "← back", BackButton);
        }
        match model {
            PageModel::Peer(m) => {
                r.spawn(widgets::value_text("Peer".into()));
                widgets::chip(r, m.id.as_bytes(), Page::Peer(m.id));
                if m.is_self {
                    r.spawn(widgets::dim_text("(self)"));
                }
                r.spawn(widgets::dim_text(if m.connected {
                    "connected"
                } else {
                    "not connected"
                }));
                match &m.did {
                    Some(did) => r.spawn(widgets::value_text(did.clone())),
                    None => r.spawn(widgets::dim_text("(no proven did)")),
                };
                r.spawn(widgets::value_text(format!("{:?}", m.trust)));
                if !m.is_self && m.did.is_some() {
                    for trust in [Trust::Trusted, Trust::Blocked] {
                        let (label, button) = TrustButton::new(m.id, trust, m.trust);
                        widgets::small_button(r, label, button);
                    }
                }
            }
            PageModel::Space(m) => {
                r.spawn(widgets::value_text("Space".into()));
                widgets::chip(r, m.space.as_bytes(), Page::Space(m.space));
                if m.active {
                    r.spawn(widgets::dim_text("(active)"));
                }
                if !m.joined {
                    r.spawn(widgets::dim_text("(not joined)"));
                }
            }
            PageModel::Doc(m) => {
                r.spawn(widgets::value_text("Doc".into()));
                widgets::chip(r, m.doc.as_bytes(), Page::Doc(m.doc));
                if m.is_space_base {
                    r.spawn(widgets::dim_text("(space base)"));
                }
            }
        }
    });
}

fn peer_page(b: &mut RelatedSpawnerCommands<ChildOf>, m: &PeerModel) {
    if m.pins.is_empty() && m.claims.is_empty() {
        b.spawn(widgets::dim_text("(no state)"));
        return;
    }

    if !m.pins.is_empty() {
        b.spawn(widgets::section_title("Pins"));
        b.spawn(widgets::grid_node(3)).with_children(|g| {
            for h in ["doc", "space", "pinned"] {
                g.spawn(widgets::header_cell(h));
            }
            for (doc, space, at) in &m.pins {
                widgets::chip(g, doc.as_bytes(), Page::Doc(*doc));
                widgets::chip(g, space.as_bytes(), Page::Space(*space));
                g.spawn((
                    widgets::AgoText(*at),
                    widgets::value_text(widgets::fmt_ago(*at)),
                ));
            }
        });
    }

    if !m.claims.is_empty() {
        b.spawn(widgets::section_title("Holds"));
        b.spawn(widgets::grid_node(2)).with_children(|g| {
            for h in ["doc", "claimed"] {
                g.spawn(widgets::header_cell(h));
            }
            for (doc, at) in &m.claims {
                widgets::chip(g, doc.as_bytes(), Page::Doc(*doc));
                g.spawn((
                    widgets::AgoText(*at),
                    widgets::value_text(widgets::fmt_ago(*at)),
                ));
            }
        });
    }
}

fn space_page(b: &mut RelatedSpawnerCommands<ChildOf>, m: &SpaceModel) {
    b.spawn(widgets::section_title("Documents"));
    if m.docs.is_empty() {
        b.spawn(widgets::dim_text("(none)"));
    } else {
        b.spawn(widgets::grid_node(4)).with_children(|g| {
            for h in ["doc", "owner", "pins", "instanced"] {
                g.spawn(widgets::header_cell(h));
            }
            for row in &m.docs {
                widgets::chip(g, row.doc.as_bytes(), Page::Doc(row.doc));
                match row.owner {
                    Some(owner) => widgets::chip(g, owner.as_bytes(), Page::Peer(owner)),
                    None => {
                        g.spawn(widgets::dim_text("space"));
                    }
                }
                g.spawn(widgets::value_text(row.pins.to_string()));
                g.spawn(widgets::dim_text(if row.instanced { "yes" } else { "-" }));
            }
        });
    }

    b.spawn(widgets::section_title("Peers"));
    if m.peers.is_empty() {
        b.spawn(widgets::dim_text("(none)"));
    } else {
        b.spawn(widgets::row_node()).with_children(|r| {
            for peer in &m.peers {
                widgets::chip(r, peer.as_bytes(), Page::Peer(*peer));
            }
        });
    }
}

fn doc_page(b: &mut RelatedSpawnerCommands<ChildOf>, m: &DocModel, expanded: &Expanded) {
    b.spawn(widgets::section_title("Info"));
    b.spawn(widgets::grid_node(2)).with_children(|g| {
        if let Some(space) = m.space {
            g.spawn(widgets::header_cell("space"));
            widgets::chip(g, space.as_bytes(), Page::Space(space));
        }
        g.spawn(widgets::header_cell("owner"));
        match m.owner {
            Some(owner) => widgets::chip(g, owner.as_bytes(), Page::Peer(owner)),
            None => {
                g.spawn(widgets::dim_text("space"));
            }
        }
        g.spawn(widgets::header_cell("holder"));
        match m.holder {
            Some(holder) => widgets::chip(g, holder.as_bytes(), Page::Peer(holder)),
            None => {
                g.spawn(widgets::dim_text("-"));
            }
        }
        if let Some(parent) = m.parent {
            g.spawn(widgets::header_cell("parent"));
            widgets::chip(g, parent.as_bytes(), Page::Doc(parent));
        }
        g.spawn(widgets::header_cell("instanced"));
        g.spawn(widgets::dim_text(if m.instanced { "yes" } else { "no" }));
        if let Some(prims) = m.prims {
            g.spawn(widgets::header_cell("prims"));
            g.spawn(widgets::value_text(prims.to_string()));
        }
    });

    b.spawn(widgets::section_title("Pinned by"));
    if m.pinned_by.is_empty() {
        b.spawn(widgets::dim_text("(no one)"));
    } else {
        b.spawn(widgets::grid_node(2)).with_children(|g| {
            for h in ["peer", "since"] {
                g.spawn(widgets::header_cell(h));
            }
            for (peer, at) in &m.pinned_by {
                widgets::chip(g, peer.as_bytes(), Page::Peer(*peer));
                g.spawn((
                    widgets::AgoText(*at),
                    widgets::value_text(widgets::fmt_ago(*at)),
                ));
            }
        });
    }

    if !m.subdocs.is_empty() {
        b.spawn(widgets::section_title("Subdocuments"));
        b.spawn(widgets::row_node()).with_children(|r| {
            for doc in &m.subdocs {
                widgets::chip(r, doc.as_bytes(), Page::Doc(*doc));
            }
        });
    }

    if !m.session.is_empty() {
        b.spawn(widgets::section_title("Session"));
        b.spawn(widgets::grid_node(6)).with_children(|g| {
            for h in ["prim", "key", "writer", "size", "written", ""] {
                g.spawn(widgets::header_cell(h));
            }
            for row in &m.session {
                g.spawn(widgets::value_text(row.prim.to_string()));
                g.spawn(widgets::value_text(row.name.to_string()));
                g.spawn(widgets::row_node()).with_children(|w| {
                    widgets::chip(w, row.writer.as_bytes(), Page::Peer(row.writer));
                });
                cell_value_cells(
                    g,
                    m.doc,
                    row.name.as_str(),
                    row.at,
                    row.value.as_deref(),
                    expanded,
                    6,
                );
            }
        });
    }

    if let Some(tree) = &m.tree {
        b.spawn(widgets::section_title("HSD"));
        b.spawn(widgets::mono_block(tree.clone()));
    }
}

/// The shared tail of a session row: size, written-ago, view toggle, and the
/// expanded value detail spanning `cols`.
fn cell_value_cells(
    g: &mut RelatedSpawnerCommands<ChildOf>,
    doc: NamespaceId,
    key: &str,
    at: u64,
    value: Option<&[u8]>,
    expanded: &Expanded,
    cols: usize,
) {
    let Some(value) = value else {
        g.spawn(widgets::dim_text("blocked"));
        g.spawn((
            widgets::AgoText(at),
            widgets::value_text(widgets::fmt_ago(at)),
        ));
        g.spawn(Node::default());
        return;
    };
    g.spawn(widgets::value_text(widgets::fmt_size(value.len())));
    g.spawn((
        widgets::AgoText(at),
        widgets::value_text(widgets::fmt_ago(at)),
    ));
    let open = expanded.0.contains(&(doc, key.to_string()));
    widgets::small_button(
        g,
        if open { "hide" } else { "view" },
        ExpandButton {
            doc,
            key: key.to_string(),
        },
    );
    if open {
        g.spawn(widgets::value_detail(value, cols));
    }
}
