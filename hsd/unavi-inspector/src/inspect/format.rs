//! How rows and values read: the row constructors, a document as a coloured
//! dot and its name or short id, and where a value comes from as a dot.

use wired_guest::{
    color::{
        document_color,
        short_id,
    },
    math::Color,
};

use crate::{
    unavi::vui::api::{
        Row,
        RowKind,
    },
    wired::{
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
    },
};

/// The dot of a value this peer states for itself only.
const RUNTIME: Color = Color::rgb(0.55, 0.6, 0.7);
/// The dot of a value this peer states for everyone, live.
const OWN_SESSION: Color = Color::rgb(0.35, 0.78, 0.5);
/// The dot of a value another peer states, live.
const PEER_SESSION: Color = Color::rgb(0.66, 0.5, 0.9);
/// The dot of a value a reference site overrides.
const OVERRIDE: Color = Color::rgb(0.95, 0.7, 0.25);

pub fn row(id: u32, label: &str, kind: RowKind) -> Row {
    Row {
        id,
        label: clip(label, 64),
        kind,
        dot: None,
        mark: None,
    }
}

pub fn header(id: u32, label: &str) -> Row {
    row(id, label, RowKind::Header)
}

pub fn action(id: u32, label: &str) -> Row {
    row(id, label, RowKind::Action)
}

pub fn field(id: u32, label: &str, value: &str) -> Row {
    row(id, label, RowKind::Field(clip(value, 128)))
}

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

/// A document as a row names it: its name, else its short id, beside a dot
/// in its own colour.
pub fn document(doc: &Document) -> (String, Color) {
    let label = match meta(doc.id()) {
        Ok(Some(meta)) => meta.name.filter(|name| !name.is_empty()),
        Ok(None) | Err(_) => None,
    };
    (
        label.unwrap_or_else(|| short_id(doc.id())),
        document_color(doc.id()),
    )
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

/// The dot saying where a value comes from, or `None` for the document
/// itself, as most values do.
pub const fn provenance(source: Option<&LayerSource>) -> Option<Color> {
    match source {
        Some(LayerSource::Override) => Some(OVERRIDE),
        Some(LayerSource::Runtime) => Some(RUNTIME),
        Some(LayerSource::OwnSession) => Some(OWN_SESSION),
        Some(LayerSource::Session(_)) => Some(PEER_SESSION),
        Some(LayerSource::Document) | None => None,
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
    fn a_value_the_document_states_has_no_dot() {
        assert_eq!(provenance(Some(&LayerSource::Document)), None);
        assert_eq!(provenance(None), None);
        assert!(provenance(Some(&LayerSource::Runtime)).is_some());
    }
}
