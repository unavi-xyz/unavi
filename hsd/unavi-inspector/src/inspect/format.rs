//! How values read: a document by its name or short id, who authors it, and
//! which layer a value comes from.

use wired_guest::color::short_id;

use crate::wired::{
    peer::authority::{
        author,
        is_author,
    },
    perception::perception::meta,
    scene::{
        document::{
            Document,
            LayerSource,
        },
        properties::{
            Property,
            PropertyKey,
        },
    },
};

/// Characters of a peer's endpoint id a layer is named by.
const PEER_TAIL: usize = 6;

/// `text` cut to at most `max` bytes, at a character boundary.
pub fn clip(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_owned();
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

/// A prim's name, or `None` when it has none.
pub fn name(doc: &Document, prim: (u64, u64)) -> Option<String> {
    match doc.get(prim, &PropertyKey::Name) {
        Some(Property::Name(name)) if !name.is_empty() => Some(name),
        _ => None,
    }
}

/// A document as it is named: its name, else its short id.
pub fn document(doc: &Document) -> String {
    let name = match meta(doc.id()) {
        Ok(Some(meta)) => meta.name.filter(|name| !name.is_empty()),
        Ok(None) | Err(_) => None,
    };
    name.unwrap_or_else(|| short_id(doc.id()))
}

pub fn author_of(doc: &Document) -> String {
    if is_author(doc).unwrap_or(false) {
        return "you".to_owned();
    }
    match author(doc) {
        Ok(Some(did)) => {
            let tail = did.len().saturating_sub(6);
            format!("…{}", &did[tail..])
        }
        Ok(None) => "space".to_owned(),
        Err(_) => "?".to_owned(),
    }
}

/// The name of the layer a value comes from.
pub fn layer_name(source: &LayerSource) -> String {
    match source {
        LayerSource::Document => "Document".to_owned(),
        LayerSource::Override => "Override".to_owned(),
        LayerSource::Runtime => "Runtime".to_owned(),
        LayerSource::OwnSession => "Your session".to_owned(),
        LayerSource::Session(peer) => {
            let start = peer
                .char_indices()
                .rev()
                .nth(PEER_TAIL - 1)
                .map_or(0, |(index, _)| index);
            format!("Peer …{}", &peer[start..])
        }
    }
}

/// Where a layer sorts in a list of them: the document's own content first.
pub const fn layer_rank(source: &LayerSource) -> u8 {
    match source {
        LayerSource::Document => 0,
        LayerSource::Override => 1,
        LayerSource::Runtime => 2,
        LayerSource::OwnSession => 3,
        LayerSource::Session(_) => 4,
    }
}

/// `ShaderBinding` as `shader binding`. Quoted text, a custom key's name, is
/// kept as written.
pub fn words(debug: &str) -> String {
    let mut out = String::with_capacity(debug.len() + 4);
    let mut quoted = false;
    let mut previous = ' ';
    for c in debug.chars() {
        match c {
            '"' => quoted = !quoted,
            '(' if !quoted => out.push_str(" ("),
            c if !quoted && c.is_ascii_uppercase() => {
                if previous.is_ascii_lowercase() {
                    out.push(' ');
                }
                out.push(c.to_ascii_lowercase());
            }
            c => out.push(c),
        }
        previous = c;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_reads_as_words() {
        assert_eq!(words("MeshVertices(Position)"), "mesh vertices (position)");
        assert_eq!(words("RigidBody"), "rigid body");
        assert_eq!(words("Custom(\"edit/locked\")"), "custom (edit/locked)");
    }

    #[test]
    fn a_peer_layer_is_named_by_the_end_of_its_endpoint_id() {
        let named = layer_name(&LayerSource::Session("abcdef123456".to_owned()));
        assert_eq!(named, "Peer …123456");
    }

    #[test]
    fn the_documents_own_layer_sorts_before_every_other() {
        let others = [
            LayerSource::Override,
            LayerSource::Runtime,
            LayerSource::OwnSession,
            LayerSource::Session(String::new()),
        ];
        for other in &others {
            assert!(layer_rank(&LayerSource::Document) < layer_rank(other));
        }
    }
}
