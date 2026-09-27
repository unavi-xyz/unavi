//! What a referencing document's store says about the prims of what it
//! references, carried into a realized reference.
//!
//! The realize path needs a store holding the target as well, so these stand
//! the realized reference up by hand, as `load::realize_ref` spawns it: an
//! `Hsd` child of the site prim.

use std::sync::{
    Arc,
    Mutex,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdNamespace,
    HsdPrimIndex,
    HsdSource,
};
use hsd::{
    id::{
        DocId,
        PrimId,
    },
    key,
    property::{
        Payload,
        Property,
        value::Value,
    },
    schema::{
        name::NameAttr,
        parent::ParentAttr,
        reference::LayerKey,
    },
    state::{
        HsdState,
        entry::Entry,
    },
};
use rstest::rstest;
use wds::document::Document;

use crate::common::*;

mod common;

const SITE: PrimId = PrimId([3; 16]);
const TARGET: PrimId = PrimId([7; 16]);

fn name_property(value: &str) -> Value {
    Value::Attribute(NameAttr(value.into()).encode().expect("encode"))
}

fn root(prim: PrimId) -> Entry {
    Entry::new(
        key::Key::prop(prim, &ParentAttr::NAME).to_string(),
        ParentAttr::to_wire(Some(ParentAttr::Root)),
        1,
    )
}

fn override_key() -> String {
    LayerKey {
        target: TARGET,
        name:   NameAttr::NAME,
    }
    .key(SITE)
}

/// A referenced document holding one prim named by its own author.
fn referenced() -> Arc<Mutex<HsdState>> {
    let mut state = HsdState::new();
    for entry in [
        root(TARGET),
        Entry::new(
            key::Key::prop(TARGET, &NameAttr::NAME).to_string(),
            name_property("couch").encode(),
            1,
        ),
    ] {
        state.project(&entry).expect("project");
    }
    Arc::new(Mutex::new(state))
}

fn prim_of(world: &World, doc: Entity, prim: PrimId) -> Option<Entity> {
    world.get::<HsdPrimIndex>(doc)?.0.get(&prim).copied()
}

fn name_of(world: &World, doc: Entity, prim: PrimId) -> Option<String> {
    let prim = prim_of(world, doc, prim)?;
    world.get::<Name>(prim).map(|name| name.as_str().to_owned())
}

/// Backs the context's document with a store holding the site prim, and
/// realizes a reference under that prim.
fn realized(ctx: &mut TestContext, backing: &Backing) -> (Document, Entity) {
    let doc = backing.document();
    set_entry(&doc, &root(SITE));

    let host = ctx.doc;
    ctx.app
        .world_mut()
        .entity_mut(host)
        .insert(HsdNamespace(doc.clone()));
    ctx.tick_until(|world| prim_of(world, host, SITE).is_some());

    let site = prim_of(ctx.app.world(), host, SITE).expect("site realized");
    let child = ctx
        .app
        .world_mut()
        .spawn((Hsd(referenced()), HsdSource(DocId([9; 32])), ChildOf(site)))
        .id();
    ctx.tick_until(|world| name_of(world, child, TARGET).as_deref() == Some("couch"));
    (doc, child)
}

#[rstest]
fn an_override_written_to_the_referencing_store_reaches_what_it_speaks_for(mut ctx: TestContext) {
    let backing = Backing::new();
    let (doc, child) = realized(&mut ctx, &backing);

    set_entry(
        &doc,
        &Entry::new(override_key(), name_property("recoloured").encode(), 2),
    );

    ctx.tick_until(|world| name_of(world, child, TARGET).as_deref() == Some("recoloured"));
}

#[rstest]
fn a_deleted_override_blocks_what_the_prim_says_about_itself(mut ctx: TestContext) {
    let backing = Backing::new();
    let (doc, child) = realized(&mut ctx, &backing);
    set_entry(
        &doc,
        &Entry::new(override_key(), name_property("recoloured").encode(), 2),
    );
    ctx.tick_until(|world| name_of(world, child, TARGET).as_deref() == Some("recoloured"));

    remove_key(&doc, override_key());

    ctx.tick_until(|world| {
        prim_of(world, child, TARGET).is_some() && name_of(world, child, TARGET).is_none()
    });
}
