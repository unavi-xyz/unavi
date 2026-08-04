use bevy_hsd::{
    HsdPrimIndex,
    HsdRelationships,
};
use hsd::attributes::material;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

/// A relationship and an attribute share one property namespace, so a reader
/// classifies them by the payload's tag byte rather than by the name.
#[traced_test]
#[rstest]
fn test_relationship_storage(mut ctx: TestContext) {
    let target = ctx.create_prim();
    let source = ctx.create_prim();

    ctx.set_relationship(source, material::BINDING, target);

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
    assert_eq!(rels.0.get(material::BINDING), Some(&target));

    ctx.remove_property(source, material::BINDING);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let rels = world
        .entity(source_ent)
        .get::<HsdRelationships>()
        .expect("relationships");
    assert!(
        rels.0.is_empty(),
        "relationship should be cleared: {:?}",
        rels.0
    );
}
