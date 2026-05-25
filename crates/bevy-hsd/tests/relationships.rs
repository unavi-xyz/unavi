use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_hsd::{
    HsdPrimIndex,
    HsdRelationships,
};
use hsd::{
    HSD_CONTAINER_ID,
    PrimMeta,
};
use loro_surgeon::{
    Reconcile,
    reconcile::RootReconciler,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_relationship_storage(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);

    let target = tree.create(None).expect("create target");
    let source = tree.create(None).expect("create source");
    let source_meta = tree.get_meta(source).expect("get meta");

    let prim = PrimMeta {
        relationships: Some(BTreeMap::from([(
            "material".to_string(),
            target.to_string(),
        )])),
        ..Default::default()
    };
    prim.reconcile(RootReconciler::new(source_meta.clone()))
        .expect("reconcile");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();

    let index = world
        .query::<&HsdPrimIndex>()
        .single(world)
        .expect("prim index");
    let source_ent = *index.0.get(&source).expect("source in index");
    let target_ent = *index.0.get(&target).expect("target in index");
    assert_ne!(source_ent, target_ent);

    let rels = world
        .entity(source_ent)
        .get::<HsdRelationships>()
        .expect("relationships");
    assert_eq!(rels.0.get("material"), Some(&target));

    let source_rel_map = source_meta
        .get("relationships")
        .and_then(|v| match v {
            loro::ValueOrContainer::Container(loro::Container::Map(m)) => Some(m),
            _ => None,
        })
        .expect("relationships map");
    source_rel_map.delete("material").expect("delete rel");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let has_rels = world.entity(source_ent).get::<HsdRelationships>().is_some();
    assert!(!has_rels, "relationships component should be removed");
}
