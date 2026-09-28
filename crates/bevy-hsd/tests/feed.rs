//! A document's store reaching its state and the world through its feed.

use bevy::prelude::*;
use bevy_hsd::{
    HsdNamespace,
    HsdPrimIndex,
};
use hsd::{
    attributes::{
        name::NameAttr,
        parent::ParentAttr,
    },
    bounds::MAX_ENTRY_BYTES,
    id::PrimId,
    key,
    property::{
        Payload,
        Property,
        value::Value,
    },
    state::entry::Entry,
};
use rstest::rstest;
use wds::document::Document;

use crate::common::*;

mod common;

const FIRST: PrimId = PrimId([1; 16]);
const SECOND: PrimId = PrimId([2; 16]);

fn root(prim: PrimId) -> Entry {
    Entry::new(
        key::Key::prop(prim, &ParentAttr::NAME).to_string(),
        ParentAttr::to_wire(Some(ParentAttr::Root)),
        1,
    )
}

fn name_value(value: &str) -> Vec<u8> {
    Value::Attribute(NameAttr(value.into()).encode().expect("encode").into()).encode()
}

fn name(prim: PrimId, value: &str) -> Entry {
    Entry::new(
        key::Key::prop(prim, &NameAttr::NAME).to_string(),
        name_value(value),
        1,
    )
}

fn is_realized(world: &World, doc: Entity, prim: PrimId) -> bool {
    world
        .get::<HsdPrimIndex>(doc)
        .is_some_and(|index| index.0.contains_key(&prim))
}

fn attach(ctx: &mut TestContext, backing: &Backing) -> Document {
    let doc = backing.document();
    ctx.app
        .world_mut()
        .entity_mut(ctx.doc)
        .insert(HsdNamespace(doc.clone()));
    doc
}

#[rstest]
fn an_entry_written_after_the_document_loaded_reaches_the_world(mut ctx: TestContext) {
    let backing = Backing::new();
    let doc = attach(&mut ctx, &backing);
    set_entry(&doc, &root(FIRST));
    let host = ctx.doc;
    ctx.tick_until(|world| is_realized(world, host, FIRST));

    set_entry(&doc, &name(FIRST, "arrived later"));

    ctx.tick_until(|world| {
        world
            .get::<HsdPrimIndex>(host)
            .and_then(|index| index.0.get(&FIRST))
            .and_then(|prim| world.get::<Name>(*prim))
            .is_some_and(|name| name.as_str() == "arrived later")
    });
}

#[rstest]
fn an_entry_over_the_size_cap_is_never_projected(mut ctx: TestContext) {
    let backing = Backing::new();
    let doc = attach(&mut ctx, &backing);
    set_entry(&doc, &root(FIRST));

    let oversized = name_value(&"x".repeat(MAX_ENTRY_BYTES));
    let size = u64::try_from(oversized.len()).expect("size");
    let hash = backing.add_bytes(oversized);
    set_hash(
        &doc,
        key::Key::prop(FIRST, &NameAttr::NAME).to_string(),
        hash,
        size,
    );
    set_entry(&doc, &root(SECOND));

    let host = ctx.doc;
    ctx.tick_until(|world| is_realized(world, host, SECOND));
    assert!(
        ctx.state
            .lock()
            .expect("lock state")
            .attribute::<NameAttr>(FIRST)
            .is_none(),
        "an entry past the cap reads as empty even though its content is held"
    );
}
