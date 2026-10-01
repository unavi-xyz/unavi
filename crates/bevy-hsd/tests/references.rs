use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_hsd::{
    document::HsdDocId,
    package::{
        ImportPackage,
        PackageAsset,
    },
    reference::ReferenceInstance,
};
use bevy_iroh::store::LocalStore;
use hsd::{
    attributes::reference::ReferenceAttr,
    format::package::Package,
    id::DocId,
};
use iroh_docs::NamespaceId;
use rstest::rstest;
use unavi_util::UtilPlugin;

use crate::common::*;

mod common;

fn with_store(ctx: &mut TestContext) -> Backing {
    let backing = Backing::new();
    ctx.app.add_plugins(UtilPlugin);
    ctx.app.world_mut().spawn(LocalStore(backing.store()));
    backing
}

#[rstest]
fn test_reference_target_is_served(mut ctx: TestContext) {
    let backing = with_store(&mut ctx);
    ctx.app
        .world_mut()
        .entity_mut(ctx.doc)
        .insert(HsdDocId(DocId([0; 32])));

    let target = backing.document().id();
    let target_id = DocId(*target.as_bytes());
    assert!(!backing.is_served(target));

    let root = ctx.create_prim();
    ctx.set_attr(root, &ReferenceAttr(target_id));

    ctx.tick_until(move |world| {
        world
            .query::<&ReferenceInstance>()
            .iter(world)
            .any(|instance| instance.target == target_id)
    });
    assert!(backing.is_served(target));
}

#[rstest]
fn test_instanced_sub_documents_are_served(mut ctx: TestContext) {
    let backing = with_store(&mut ctx);
    let before = backing.namespaces();

    let mut package = Package::new(BTreeMap::new());
    package.documents.push((DocId([7; 32]), Vec::new()));
    let handle = ctx
        .app
        .world_mut()
        .resource_mut::<Assets<PackageAsset>>()
        .add(PackageAsset(package));
    let instance = ctx.app.world_mut().spawn(ImportPackage(handle)).id();

    ctx.tick_until(move |world| world.get::<HsdDocId>(instance).is_some());
    let root = NamespaceId::from(
        &ctx.app
            .world()
            .get::<HsdDocId>(instance)
            .expect("instanced")
            .0
            .0,
    );

    let subs: Vec<_> = backing
        .namespaces()
        .into_iter()
        .filter(|ns| *ns != root && !before.contains(ns))
        .collect();
    assert_eq!(subs.len(), 1);
    assert!(backing.is_served(subs[0]));
}
