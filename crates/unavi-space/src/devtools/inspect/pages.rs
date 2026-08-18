use bevy::{
    ecs::relationship::RelatedSpawnerCommands,
    prelude::*,
};
use iroh_docs::NamespaceId;

use crate::devtools::inspect::{
    BackButton,
    ExpandButton,
    Expanded,
    Page,
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
        PageModel::Peer(m) => peer_page(b, m, expanded),
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
                widgets::chip(r, &m.id, Page::Peer(m.id));
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

fn peer_page(b: &mut RelatedSpawnerCommands<ChildOf>, m: &PeerModel, expanded: &Expanded) {
    if m.pins.is_empty() && m.claims.is_empty() && m.kv.is_empty() {
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
        b.spawn(widgets::section_title("Authority"));
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

    if !m.kv.is_empty() {
        b.spawn(widgets::section_title("Key-Value"));
        b.spawn(widgets::grid_node(5)).with_children(|g| {
            for h in ["doc", "key", "size", "written", ""] {
                g.spawn(widgets::header_cell(h));
            }
            for row in &m.kv {
                widgets::chip(g, row.doc.as_bytes(), Page::Doc(row.doc));
                g.spawn(widgets::value_text(row.key.clone()));
                kv_value_cells(
                    g,
                    row.doc,
                    &row.key,
                    row.at,
                    row.value.as_deref(),
                    expanded,
                    5,
                );
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
                    Some(owner) => widgets::chip(g, &owner, Page::Peer(owner)),
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
                widgets::chip(r, peer, Page::Peer(*peer));
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
            Some(owner) => widgets::chip(g, &owner, Page::Peer(owner)),
            None => {
                g.spawn(widgets::dim_text("space"));
            }
        }
        g.spawn(widgets::header_cell("authority"));
        match m.authority {
            Some(authority) => widgets::chip(g, &authority, Page::Peer(authority)),
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
                widgets::chip(g, peer, Page::Peer(*peer));
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

    if !m.kv.is_empty() {
        b.spawn(widgets::section_title("Key-Value"));
        b.spawn(widgets::grid_node(5)).with_children(|g| {
            for h in ["key", "writer", "size", "written", ""] {
                g.spawn(widgets::header_cell(h));
            }
            for row in &m.kv {
                g.spawn(widgets::value_text(row.key.clone()));
                g.spawn(widgets::row_node()).with_children(|w| {
                    widgets::chip(w, &row.writer, Page::Peer(row.writer));
                    if row.neutral {
                        w.spawn(widgets::dim_text("neutral"));
                    }
                });
                kv_value_cells(
                    g,
                    m.doc,
                    &row.key,
                    row.at,
                    row.value.as_deref(),
                    expanded,
                    5,
                );
            }
        });
    }

    if let Some(tree) = &m.tree {
        b.spawn(widgets::section_title("HSD"));
        b.spawn(widgets::mono_block(tree.clone()));
    }
}

/// The shared tail of a KV row: size, written-ago, view toggle, and the
/// expanded value detail spanning `cols`.
fn kv_value_cells(
    g: &mut RelatedSpawnerCommands<ChildOf>,
    doc: NamespaceId,
    key: &str,
    at: u64,
    value: Option<&[u8]>,
    expanded: &Expanded,
    cols: usize,
) {
    let Some(value) = value else {
        g.spawn(widgets::dim_text("tombstone"));
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
