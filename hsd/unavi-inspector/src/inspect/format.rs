//! How rows and values read: the row constructors, and a property or a key as a
//! person reads it.

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
            document::Document,
            properties::{
                Property,
                PropertyKey,
            },
        },
    },
};

pub fn header(id: u32, label: &str) -> Row {
    Row {
        id,
        label: clip(label, 64),
        kind: RowKind::Header,
    }
}

pub fn action(id: u32, label: &str) -> Row {
    Row {
        id,
        label: clip(label, 64),
        kind: RowKind::Action,
    }
}

pub fn field(id: u32, label: &str, value: &str) -> Row {
    Row {
        id,
        label: clip(label, 64),
        kind: RowKind::Field(clip(value, 128)),
    }
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

pub fn name(doc: &Document, prim: (u64, u64)) -> String {
    match doc.get(prim, &PropertyKey::Name) {
        Some(Property::Name(name)) => name,
        _ => "Unnamed prim".to_owned(),
    }
}

pub fn document_name(doc: &Document) -> String {
    match meta(doc.id()) {
        Ok(Some(meta)) => meta.name.unwrap_or_else(|| "unnamed".to_owned()),
        Ok(None) | Err(_) => "unknown".to_owned(),
    }
}

pub fn author_of(doc: &Document) -> String {
    if is_author(doc).unwrap_or(false) {
        return "you".to_owned();
    }
    match author(doc) {
        Ok(Some(did)) => {
            let tail = did.len().saturating_sub(8);
            format!("…{}", &did[tail..])
        }
        Ok(None) => "the space".to_owned(),
        Err(_) => "unknown".to_owned(),
    }
}

/// `MeshVertices(Position)` as `mesh vertices (position)`. Quoted text, a
/// custom key's name, is kept as written.
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
}
