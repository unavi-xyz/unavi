//! What a referencing document says about the prims of what it references.
//!
//! The realize path needs a document store, so these stand a realized
//! reference up by hand — an `Hsd` child of the site prim, as
//! `load::realize_ref` spawns it — and exercise what carries an override
//! across to it afterwards.

use std::sync::{
    Arc,
    Mutex,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdSource,
    load::RefOverrides,
};
use hsd::{
    attributes::{
        Attribute,
        name::NameAttr,
    },
    id::{
        DocId,
        PrimId,
    },
    key,
    property::{
        Parent,
        Property,
    },
    state::{
        HsdState,
        entry::Entry,
    },
};
use rstest::rstest;

use crate::common::*;

mod common;

const TARGET: PrimId = PrimId([7; 16]);

fn name_property(value: &str) -> Property {
    Property::Attribute(NameAttr(value.into()).encode().expect("encode"))
}

/// A referenced document holding one prim named by its own author.
fn referenced() -> Arc<Mutex<HsdState>> {
    let mut state = HsdState::new();
    state
        .apply_all(&[
            Entry::new(key::parent(TARGET), Parent::Root.encode(), 1),
            Entry::new(
                key::prop(TARGET, NameAttr::KEY),
                name_property("couch").encode(),
                1,
            ),
        ])
        .expect("apply");
    Arc::new(Mutex::new(state))
}

/// Stands a realized reference up under `site`, as the realizer does.
fn realize(ctx: &mut TestContext, site: Entity, state: &Arc<Mutex<HsdState>>) -> Entity {
    ctx.app
        .world_mut()
        .spawn((
            Hsd(Arc::clone(state)),
            HsdSource(DocId([9; 32])),
            RefOverrides(0),
            ChildOf(site),
        ))
        .id()
}

fn name_of(ctx: &TestContext, doc: Entity, prim: PrimId) -> Option<String> {
    let prim = ctx.prim_entity(doc, prim);
    ctx.app
        .world()
        .get::<Name>(prim)
        .map(|name| name.as_str().to_owned())
}

#[rstest]
fn an_override_written_after_the_fact_reaches_what_it_speaks_for(mut ctx: TestContext) {
    let site = ctx.create_prim();
    ctx.app.update();
    let site_ent = ctx.prim_entity(ctx.doc, site);

    let state = referenced();
    let child = realize(&mut ctx, site_ent, &state);
    ctx.app.update();
    assert_eq!(name_of(&ctx, child, TARGET).as_deref(), Some("couch"));

    ctx.apply(&Entry::new(
        key::ref_layer_key(site, TARGET, NameAttr::KEY),
        name_property("recoloured").encode(),
        2,
    ));
    ctx.app.update();

    assert_eq!(
        name_of(&ctx, child, TARGET).as_deref(),
        Some("recoloured"),
        "an override is durable in the referencing document, and this is what \
         carries it to the one it speaks for"
    );
}

#[rstest]
fn a_blocked_override_hides_what_the_prim_says_about_itself(mut ctx: TestContext) {
    let site = ctx.create_prim();
    ctx.app.update();
    let site_ent = ctx.prim_entity(ctx.doc, site);

    let state = referenced();
    let child = realize(&mut ctx, site_ent, &state);
    ctx.apply(&Entry::new(
        key::ref_layer_key(site, TARGET, NameAttr::KEY),
        name_property("recoloured").encode(),
        2,
    ));
    ctx.app.update();
    assert_eq!(name_of(&ctx, child, TARGET).as_deref(), Some("recoloured"));

    // An empty value under the same key: the referencing document states that
    // the property is gone rather than stating one of its own.
    ctx.apply(&Entry::new(
        key::ref_layer_key(site, TARGET, NameAttr::KEY),
        Vec::new(),
        3,
    ));
    ctx.app.update();

    assert_eq!(
        name_of(&ctx, child, TARGET).as_deref(),
        None,
        "an empty value blocks the key rather than falling through, which is \
         how a room hides a prim's property without editing the prim"
    );
}
