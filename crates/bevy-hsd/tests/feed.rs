//! A document's store reaching its state and the world through its feed.

use std::time::Duration;

use bevy::prelude::*;
use bevy_hsd::{
    document::HsdNamespace,
    feed::{
        DocFeed,
        FeedReady,
    },
    prim::PrimIndex,
};
use hsd::{
    attributes::{
        name::NameAttr,
        parent::ParentAttr,
    },
    id::PrimId,
    key,
    property::{
        Payload,
        Property,
        value::Value,
    },
    state::entry::Entry,
};
use iroh_blobs::Hash;
use rstest::rstest;
use unavi_store::{
    Document,
    MAX_ENTRY_BYTES,
};

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

fn is_in_scene(world: &World, doc: Entity, prim: PrimId) -> bool {
    world
        .get::<PrimIndex>(doc)
        .is_some_and(|index| index.get(prim).is_some())
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
    ctx.tick_until(|world| is_in_scene(world, host, FIRST));

    set_entry(&doc, &name(FIRST, "arrived later"));

    ctx.tick_until(|world| {
        world
            .get::<PrimIndex>(host)
            .and_then(|index| index.get(FIRST))
            .and_then(|prim| world.get::<Name>(prim))
            .is_some_and(|name| name.as_str() == "arrived later")
    });
}

#[rstest]
fn an_entry_over_the_size_cap_is_never_projected(mut ctx: TestContext) {
    let backing = Backing::new();
    let doc = attach(&mut ctx, &backing);
    set_entry(&doc, &root(FIRST));

    let oversized = name_value(&"x".repeat(usize::try_from(MAX_ENTRY_BYTES).expect("cap")));
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
    ctx.tick_until(|world| is_in_scene(world, host, SECOND));
    assert!(
        ctx.state
            .lock()
            .expect("lock state")
            .attribute::<NameAttr>(FIRST)
            .is_none(),
        "an entry past the cap reads as empty even though its content is held"
    );
}

/// Feeds `doc` into the context's document with `ready` and `deadline`.
fn attach_feed(ctx: &mut TestContext, doc: &Document, ready: FeedReady, deadline: Duration) {
    ctx.app
        .world_mut()
        .entity_mut(ctx.doc)
        .insert(DocFeed::spawn_with_deadline(doc.clone(), ready, deadline));
    ctx.app
        .world_mut()
        .entity_mut(ctx.doc)
        .insert(HsdNamespace(doc.clone()));
}

fn is_synced(world: &World, doc: Entity) -> bool {
    world.get::<DocFeed>(doc).is_some_and(DocFeed::is_synced)
}

/// An entry whose content no peer has sent and the store does not hold.
fn set_missing_content(doc: &Document, prim: PrimId) {
    let bytes = name_value("never downloaded");
    let size = u64::try_from(bytes.len()).expect("size");
    let hash = Hash::new(&bytes);
    set_hash(
        doc,
        key::Key::prop(prim, &NameAttr::NAME).to_string(),
        hash,
        size,
    );
}

#[rstest]
fn a_feed_is_not_synced_while_a_snapshot_entry_has_no_content(mut ctx: TestContext) {
    let backing = Backing::new();
    let doc = backing.document();
    set_entry(&doc, &root(FIRST));
    set_missing_content(&doc, FIRST);
    set_entry(&doc, &root(SECOND));
    attach_feed(
        &mut ctx,
        &doc,
        FeedReady::Snapshot,
        Duration::from_secs(3600),
    );

    let host = ctx.doc;
    ctx.tick_until(|world| is_in_scene(world, host, SECOND));
    for _ in 0..10 {
        ctx.app.update();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        !is_synced(ctx.app.world(), host),
        "synced was sent with a snapshot entry's content missing"
    );
}

#[rstest]
fn a_feed_with_nothing_missing_is_synced_after_its_snapshot(mut ctx: TestContext) {
    let backing = Backing::new();
    let doc = backing.document();
    set_entry(&doc, &root(FIRST));
    attach_feed(
        &mut ctx,
        &doc,
        FeedReady::Snapshot,
        Duration::from_secs(3600),
    );

    let host = ctx.doc;
    ctx.tick_until(|world| is_synced(world, host));
}

#[rstest]
fn a_feed_is_synced_at_its_deadline_when_content_never_arrives(mut ctx: TestContext) {
    let backing = Backing::new();
    let doc = backing.document();
    set_entry(&doc, &root(FIRST));
    set_missing_content(&doc, FIRST);
    attach_feed(
        &mut ctx,
        &doc,
        FeedReady::RemoteSync,
        Duration::from_millis(100),
    );

    let host = ctx.doc;
    ctx.tick_until(|world| is_synced(world, host));
}
