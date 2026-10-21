//! One inspected prim: what the slate says about it, and the edits its rows
//! make. The rows edit the prim itself, its transform among them; how the
//! element it belongs to is placed is read from what it stands for as a whole
//! (a referenced document's site, a prim of the space, a loose document's
//! root).

use std::fmt::Write;

use wired_guest::math::Quat;

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
                LayerSource,
                copy_document,
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

/// Most attribute and child rows, so the rest of the slate always fits.
const MAX_ATTRIBUTES: usize = 6;
const MAX_CHILDREN: usize = 3;
/// The smallest scale the Scale row brings an element down to.
const MIN_SCALE: f32 = 0.01;

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
    pub const TREE: u32 = 30;
    pub const PARENT: u32 = 31;
    /// Child rows count up from here.
    pub const CHILD: u32 = 32;
    pub const ATTRIBUTES: u32 = 20;
    /// Attribute rows count up from here.
    pub const ATTRIBUTE: u32 = 21;
    pub const MAKE_LOCAL: u32 = 49;
    pub const CLOSE: u32 = 50;
}

/// What a row press asked of the tool rather than of the element.
pub enum Asked {
    Nothing,
    Close,
    /// Inspect this prim instead: a parent or a child chosen from the rows.
    Inspect(PrimRef),
}

pub struct Inspection {
    /// The document holding the prim the hit stands for as a whole.
    placed_in: Document,
    loose:     bool,
    /// The reference site in `placed_in` placing the hit's document, when a
    /// reference does.
    site:      Option<(u64, u64)>,
    hit_doc:   Document,
    hit_prim:  (u64, u64),
    /// The transform edit a drag on a number row has open.
    edit:      Option<Edit>,
}

impl Inspection {
    /// Inspects what a hit on `target` stands for, or `None` when a document
    /// it needs is not loaded.
    #[must_use]
    pub fn of(target: PrimRef) -> Option<Self> {
        let hit_doc = open_document(target.document).ok()??;
        let (placed_in, prim, loose) = element_of(target)?;
        let site = (!loose && placed_in.id() != hit_doc.id()).then_some(prim);
        Some(Self {
            placed_in,
            loose,
            site,
            hit_doc,
            hit_prim: target.prim,
            edit: None,
        })
    }

    #[must_use]
    pub fn rows(&self) -> Vec<Row> {
        let mut rows = vec![
            header(row::NAME, &name(&self.hit_doc, self.hit_prim)),
            field(row::DOCUMENT, "Document", &document_name(&self.hit_doc)),
            field(row::AUTHOR, "Author", &author_of(&self.hit_doc)),
            field(row::KIND, "Is", self.kind()),
        ];

        if let Some(Property::Transform(transform)) =
            self.hit_doc.get(self.hit_prim, &PropertyKey::Transform)
        {
            let editable = self.hit_doc.may_write();
            let yaw = transform.rotation.yaw();
            rows.push(header(
                row::TRANSFORM,
                &format!("Transform · {}", self.source_of(&PropertyKey::Transform)),
            ));
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

        rows.extend(self.tree_rows());

        let keys = self.hit_doc.keys(self.hit_prim);
        rows.push(header(
            row::ATTRIBUTES,
            &format!("Attributes ({})", keys.len()),
        ));
        for (index, key) in keys.iter().take(MAX_ATTRIBUTES).enumerate() {
            let summary = self
                .hit_doc
                .get(self.hit_prim, key)
                .map(|property| summary(&property))
                .unwrap_or_default();
            let source = self.source_of(key);
            let value = if summary.is_empty() {
                source
            } else {
                format!("{summary} · {source}")
            };
            rows.push(field(
                row::ATTRIBUTE + index as u32,
                &key_label(key),
                &value,
            ));
        }
        if self.site.is_some() && self.placed_in.may_write() {
            rows.push(action(row::MAKE_LOCAL, "Make local"));
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
            // The site opens the copy in place of what it showed, so this
            // inspection's document is gone.
            SlateEvent::Pressed(row::MAKE_LOCAL) => {
                match self.make_local() {
                    Ok(()) => println!("inspector: made local"),
                    Err(err) => eprintln!("inspector: make local: {err}"),
                }
                return Asked::Close;
            }
            SlateEvent::Pressed(row::PARENT) => {
                if let Some(parent) = self.parent() {
                    return Asked::Inspect(self.at(parent));
                }
            }
            SlateEvent::Pressed(id)
                if (row::CHILD..row::CHILD + MAX_CHILDREN as u32).contains(&id) =>
            {
                let index = (id - row::CHILD) as usize;
                if let Some(child) = self.hit_doc.children(self.hit_prim).get(index) {
                    return Asked::Inspect(self.at(*child));
                }
            }
            SlateEvent::Scrubbed((row, by)) => {
                if let Err(err) = self.scrub(row, by) {
                    eprintln!("inspector: edit: {err}");
                }
            }
            SlateEvent::Released(_) => self.finish(),
            SlateEvent::Pressed(_) | SlateEvent::Toggled(_) => {}
        }
        Asked::Nothing
    }

    /// Which layer `key`'s value comes from, as a person reads it.
    fn source_of(&self, key: &PropertyKey) -> String {
        match self.hit_doc.source(self.hit_prim, key) {
            Some(LayerSource::Document) => "document".to_owned(),
            Some(LayerSource::Override) => "override".to_owned(),
            Some(LayerSource::Runtime) => "this peer only".to_owned(),
            Some(LayerSource::OwnSession) => "you, live".to_owned(),
            Some(LayerSource::Session(peer)) => {
                let skip = peer.chars().count().saturating_sub(4);
                format!("peer …{}", peer.chars().skip(skip).collect::<String>())
            }
            None => "unset".to_owned(),
        }
    }

    /// The hit prim's parent and first children, each a row that inspects it.
    fn tree_rows(&self) -> Vec<Row> {
        let parent = self.parent();
        let children = self.hit_doc.children(self.hit_prim);
        if parent.is_none() && children.is_empty() {
            return Vec::new();
        }
        let mut rows = vec![header(
            row::TREE,
            &format!("Tree ({} beneath)", children.len()),
        )];
        if let Some(parent) = parent {
            rows.push(action(
                row::PARENT,
                &format!("Up to {}", name(&self.hit_doc, parent)),
            ));
        }
        for (index, child) in children.iter().take(MAX_CHILDREN).enumerate() {
            rows.push(action(
                row::CHILD + index as u32,
                &format!("Into {}", name(&self.hit_doc, *child)),
            ));
        }
        rows
    }

    fn parent(&self) -> Option<(u64, u64)> {
        match self.hit_doc.get(self.hit_prim, &PropertyKey::Parent) {
            Some(Property::Parent(parent)) => parent,
            _ => None,
        }
    }

    /// `prim` of the inspected document, as a hit on it would name it.
    fn at(&self, prim: (u64, u64)) -> PrimRef {
        PrimRef {
            document: self.hit_doc.id(),
            prim,
        }
    }

    /// Replaces the reference with a copy of what it shows, undoably, so the
    /// object here stops following its source. Overrides kept on the site
    /// still apply, since they are keyed by the site.
    fn make_local(&self) -> anyhow::Result<()> {
        let site = self
            .site
            .ok_or_else(|| anyhow::anyhow!("no reference places this"))?;
        let copy = copy_document(&self.hit_doc)?;
        let edit = Edit::new("Make local")?;
        edit.track(&self.placed_in, &[(site, PropertyKey::Reference)]);
        self.placed_in
            .local()
            .set(site, Property::Reference(copy.id()))
            .flush()?;
        edit.commit()?;
        Ok(())
    }

    /// Keeps the transform edit a drag left open, so ending an inspection
    /// mid-drag still files it.
    pub fn finish(&mut self) {
        if let Some(edit) = self.edit.take()
            && let Err(err) = edit.commit()
        {
            eprintln!("inspector: keep: {err:?}");
        }
    }

    fn kind(&self) -> &'static str {
        if self.loose {
            "loose: here while someone holds it"
        } else if self.placed_in.id() == self.hit_doc.id() {
            "a prim of this space"
        } else {
            "placed by a reference"
        }
    }

    fn scrub(&mut self, row: u32, by: f32) -> anyhow::Result<()> {
        let Some(Property::Transform(mut transform)) =
            self.hit_doc.get(self.hit_prim, &PropertyKey::Transform)
        else {
            return Ok(());
        };
        if self.edit.is_none() {
            let edit = Edit::new("Edit transform")?;
            edit.track(&self.hit_doc, &[(self.hit_prim, PropertyKey::Transform)]);
            self.edit = Some(edit);
        }
        match row {
            row::X => transform.translation.x += by,
            row::Y => transform.translation.y += by,
            row::Z => transform.translation.z += by,
            row::YAW => {
                transform.rotation = Quat::from_rotation_y(by.to_radians()) * transform.rotation;
            }
            // Scaled as a whole, keeping the proportions a stretched element
            // has; the row reads its X.
            row::SCALE => {
                let x = transform.scale.x;
                if x.abs() > f32::EPSILON {
                    transform.scale *= ((x + by) / x).max(MIN_SCALE / x.abs());
                }
            }
            _ => return Ok(()),
        }
        let set = Property::Transform(transform);
        if let Err(err) = self
            .hit_doc
            .shared()
            .set(self.hit_prim, set.clone())
            .flush()
        {
            eprintln!("inspector: editing for this peer only: {err:?}");
            self.hit_doc.local().set(self.hit_prim, set).flush()?;
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

fn action(id: u32, label: &str) -> Row {
    Row {
        id,
        label: clip(label, 64),
        kind: RowKind::Action,
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
