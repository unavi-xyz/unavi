//! One inspected element: what the slate says about it, and the edits its
//! rows make.
//!
//! The element is what a hit stands for as a whole (a referenced document's
//! site, a prim of the space, a loose document's root), and its transform is
//! what the number rows edit. The attributes listed are the hit prim's own.

use std::fmt::Write;

use wired_guest::math::{
    Quat,
    Vec3,
};

use crate::{
    element_of,
    unavi::{
        author::api::Edit,
        vui::api::{
            Number,
            Row,
            RowKind,
            SlateEvent,
        },
    },
    wired::{
        core::ids::PrimRef,
        peer::authority::{
            author,
            is_author,
        },
        perception::perception::meta,
        scene::{
            document::{
                Document,
                open_document,
            },
            properties::{
                Collider,
                Property,
                PropertyKey,
                RigidBodyKind,
            },
        },
    },
};

/// Most attribute rows, so the rest of the slate always fits.
const MAX_ATTRIBUTES: usize = 10;

mod row {
    pub const NAME: u32 = 0;
    pub const DOCUMENT: u32 = 1;
    pub const AUTHOR: u32 = 2;
    pub const KIND: u32 = 3;
    pub const TRANSFORM: u32 = 10;
    pub const X: u32 = 11;
    pub const Y: u32 = 12;
    pub const Z: u32 = 13;
    pub const YAW: u32 = 14;
    pub const SCALE: u32 = 15;
    pub const ATTRIBUTES: u32 = 20;
    /// Attribute rows count up from here.
    pub const ATTRIBUTE: u32 = 21;
    pub const CLOSE: u32 = 50;
}

/// What a row press asked of the tool rather than of the element.
pub enum Asked {
    Nothing,
    Close,
}

pub struct Inspection {
    doc:      Document,
    prim:     (u64, u64),
    loose:    bool,
    hit_doc:  Document,
    hit_prim: (u64, u64),
    /// The transform edit a drag on a number row has open.
    edit:     Option<Edit>,
}

impl Inspection {
    /// Inspects what a hit on `target` stands for, or `None` when a document
    /// it needs is not loaded.
    #[must_use]
    pub fn of(target: PrimRef) -> Option<Self> {
        let hit_doc = open_document(target.document).ok()??;
        let (doc, prim, loose) = element_of(target)?;
        Some(Self {
            doc,
            prim,
            loose,
            hit_doc,
            hit_prim: target.prim,
            edit: None,
        })
    }

    #[must_use]
    pub fn rows(&self) -> Vec<Row> {
        let mut rows = vec![
            header(row::NAME, &name(&self.doc, self.prim)),
            field(row::DOCUMENT, "Document", &document_name(&self.doc)),
            field(row::AUTHOR, "Author", &author_of(&self.doc)),
            field(row::KIND, "Is", self.kind()),
        ];

        if let Some(Property::Transform(transform)) =
            self.doc.get(self.prim, &PropertyKey::Transform)
        {
            let editable = self.doc.may_write();
            let yaw = transform.rotation.yaw();
            rows.push(header(row::TRANSFORM, "Transform"));
            rows.push(number(row::X, "X", transform.translation.x, 1.0, editable));
            rows.push(number(row::Y, "Y", transform.translation.y, 1.0, editable));
            rows.push(number(row::Z, "Z", transform.translation.z, 1.0, editable));
            rows.push(number(row::YAW, "Yaw°", yaw.to_degrees(), 180.0, editable));
            rows.push(number(
                row::SCALE,
                "Scale",
                transform.scale.x,
                1.0,
                editable,
            ));
        }

        let keys = self.hit_doc.keys(self.hit_prim);
        rows.push(header(
            row::ATTRIBUTES,
            &format!("Attributes ({})", keys.len()),
        ));
        for (index, key) in keys.iter().take(MAX_ATTRIBUTES).enumerate() {
            let value = self
                .hit_doc
                .get(self.hit_prim, key)
                .map(|property| summary(&property))
                .unwrap_or_default();
            rows.push(field(
                row::ATTRIBUTE + index as u32,
                &key_label(key),
                &value,
            ));
        }
        rows.push(Row {
            id:    row::CLOSE,
            label: "Close".to_owned(),
            kind:  RowKind::Action,
        });
        rows
    }

    /// Answers what a row did. Number rows edit the transform live and keep
    /// it, undoably, when the drag ends.
    pub fn apply(&mut self, event: &SlateEvent) -> Asked {
        match *event {
            SlateEvent::Pressed(row::CLOSE) => return Asked::Close,
            SlateEvent::Scrubbed((row, by)) => {
                if let Err(err) = self.scrub(row, by) {
                    eprintln!("inspector: edit: {err}");
                }
            }
            SlateEvent::Released(_) => {
                if let Some(edit) = self.edit.take()
                    && let Err(err) = edit.commit()
                {
                    eprintln!("inspector: keep: {err:?}");
                }
            }
            SlateEvent::Pressed(_) | SlateEvent::Toggled(_) => {}
        }
        Asked::Nothing
    }

    fn kind(&self) -> &'static str {
        if self.loose {
            "loose: here while someone holds it"
        } else if self.doc.id() == self.hit_doc.id() {
            "a prim of this space"
        } else {
            "placed by a reference"
        }
    }

    fn scrub(&mut self, row: u32, by: f32) -> anyhow::Result<()> {
        let Some(Property::Transform(mut transform)) =
            self.doc.get(self.prim, &PropertyKey::Transform)
        else {
            return Ok(());
        };
        if self.edit.is_none() {
            let edit = Edit::new("Edit transform")?;
            edit.track(&self.doc, &[(self.prim, PropertyKey::Transform)]);
            self.edit = Some(edit);
        }
        match row {
            row::X => transform.translation.x += by,
            row::Y => transform.translation.y += by,
            row::Z => transform.translation.z += by,
            row::YAW => {
                transform.rotation = Quat::from_rotation_y(by.to_radians()) * transform.rotation;
            }
            row::SCALE => {
                let scale = (transform.scale.x + by).max(0.01);
                transform.scale = Vec3::splat(scale);
            }
            _ => return Ok(()),
        }
        let set = Property::Transform(transform);
        if let Err(err) = self.doc.shared().set(self.prim, set.clone()).flush() {
            eprintln!("inspector: editing for this peer only: {err:?}");
            self.doc.local().set(self.prim, set).flush()?;
        }
        Ok(())
    }
}

fn header(id: u32, label: &str) -> Row {
    Row {
        id,
        label: clip(label, 64),
        kind: RowKind::Header,
    }
}

fn field(id: u32, label: &str, value: &str) -> Row {
    Row {
        id,
        label: clip(label, 64),
        kind: RowKind::Field(clip(value, 128)),
    }
}

/// A number row, or the same value read-only where the user may not edit it.
fn number(id: u32, label: &str, value: f32, per_metre: f32, editable: bool) -> Row {
    if !editable {
        return field(id, label, &format!("{value:.2}"));
    }
    Row {
        id,
        label: label.to_owned(),
        kind: RowKind::Number(Number { value, per_metre }),
    }
}

/// `text` cut to at most `max` bytes, at a character boundary.
fn clip(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_owned();
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

fn name(doc: &Document, prim: (u64, u64)) -> String {
    match doc.get(prim, &PropertyKey::Name) {
        Some(Property::Name(name)) => name,
        _ => "Unnamed prim".to_owned(),
    }
}

fn document_name(doc: &Document) -> String {
    match meta(doc.id()) {
        Ok(Some(meta)) => meta.name.unwrap_or_else(|| "unnamed".to_owned()),
        Ok(None) | Err(_) => "unknown".to_owned(),
    }
}

fn author_of(doc: &Document) -> String {
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

/// The key's name as a person reads it.
fn key_label(key: &PropertyKey) -> String {
    let mut label = String::new();
    // A key's `Debug` form is its variant, which is what the WIT names it.
    if write!(label, "{key:?}").is_err() {
        return "attribute".to_owned();
    }
    label
}

/// A short reading of a value, empty for one that is bytes to the eye.
fn summary(property: &Property) -> String {
    match property {
        Property::Name(name) => name.clone(),
        Property::Text(text) => text.value.clone(),
        Property::Reference(doc) => format!("…{:x}", doc.3 & 0xFFFF),
        Property::RigidBody(body) => match body.kind {
            RigidBodyKind::Dynamic => "dynamic".to_owned(),
            RigidBodyKind::Kinematic => "kinematic".to_owned(),
            RigidBodyKind::Static => "static".to_owned(),
        },
        Property::Collider(collider) => match collider {
            Collider::Cuboid(_) => "box".to_owned(),
            Collider::Sphere(_) => "sphere".to_owned(),
            Collider::Capsule(_) => "capsule".to_owned(),
            Collider::Cylinder(_) => "cylinder".to_owned(),
            _ => "mesh".to_owned(),
        },
        Property::GravityScale(scale) => format!("{scale:.2}"),
        Property::Material(material) => material
            .base_color
            .map_or_else(String::new, |c| format!("{:.2} {:.2} {:.2}", c.r, c.g, c.b)),
        Property::MeshIndices(indices) => format!("{} indices", indices.len()),
        Property::MeshVertices(stream) => format!("{} floats", stream.values.len()),
        _ => String::new(),
    }
}
