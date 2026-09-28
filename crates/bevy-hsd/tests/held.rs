use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdHeld,
    Prim,
    anchor::{
        self,
        DocAnchor,
    },
};
use hsd::attributes::xform::XformAttr;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

const AT: Vec3 = Vec3::new(3.0, 1.0, -2.0);

/// A held document in place of the context's live one, so the same helpers
/// build content into something that is not in the scene.
fn hold(ctx: &mut TestContext) -> Entity {
    let doc = ctx
        .app
        .world_mut()
        .query_filtered::<Entity, With<Hsd>>()
        .single(ctx.app.world())
        .expect("the context's document");
    ctx.app.world_mut().entity_mut(doc).despawn();

    ctx.app
        .world_mut()
        .spawn(HsdHeld(Arc::clone(&ctx.state)))
        .id()
}

fn prims(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), With<Prim>>()
        .iter(app.world())
        .count()
}

#[traced_test]
#[rstest]
fn a_held_document_realizes_nothing(mut ctx: TestContext) {
    let doc = hold(&mut ctx);
    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &XformAttr {
            rotation:    Quat::IDENTITY.to_array(),
            scale:       [1.0; 3],
            translation: [1.0, 2.0, 3.0],
        },
    );

    ctx.app.update();
    ctx.app.update();

    assert_eq!(prims(&mut ctx.app), 0, "a held document draws nothing");
    let world = ctx.app.world();
    assert!(
        world.get::<Transform>(doc).is_none(),
        "and is nowhere, so it cannot be at the origin"
    );
}

#[traced_test]
#[rstest]
fn placing_a_held_document_brings_all_of_it_in_at_once(mut ctx: TestContext) {
    let doc = hold(&mut ctx);
    let root = ctx.create_prim();
    ctx.create_child(root);
    ctx.create_child(root);

    // Built while held: none of it has reached the world, and the events that
    // said so were dropped rather than banked.
    ctx.app.update();
    assert_eq!(prims(&mut ctx.app), 0);

    anchor::place(
        &mut ctx.app.world_mut().entity_mut(doc),
        DocAnchor::root(Transform::from_translation(AT)),
    );
    ctx.app.update();

    assert_eq!(
        prims(&mut ctx.app),
        3,
        "everything built while held is realized by the placement"
    );
    assert_eq!(
        ctx.app.world().get::<Transform>(doc).map(|t| t.translation),
        Some(AT),
        "and stands where it was put, not at the origin"
    );
    assert!(ctx.app.world().get::<HsdHeld>(doc).is_none());
}

/// The end state only; the test app runs single-threaded, so it cannot tell
/// whether `apply_anchors` is ordered before propagation or merely happened to
/// be inserted first.
#[traced_test]
#[rstest]
fn a_placed_document_is_drawn_where_it_was_put(mut ctx: TestContext) {
    let doc = hold(&mut ctx);
    ctx.create_prim();

    anchor::place(
        &mut ctx.app.world_mut().entity_mut(doc),
        DocAnchor::root(Transform::from_translation(AT)),
    );
    ctx.app.update();

    let world = ctx.app.world_mut();
    let prim = world
        .query_filtered::<Entity, With<Prim>>()
        .single(world)
        .expect("the realized prim");
    assert_eq!(
        ctx.app
            .world()
            .get::<GlobalTransform>(prim)
            .map(GlobalTransform::translation),
        Some(AT),
        "a prim realized by the placement stands in the document's frame, not \
         at the space origin"
    );
}

#[traced_test]
#[rstest]
fn placing_again_moves_rather_than_rebuilds(mut ctx: TestContext) {
    let doc = hold(&mut ctx);
    ctx.create_prim();

    for offset in [Vec3::ZERO, AT] {
        anchor::place(
            &mut ctx.app.world_mut().entity_mut(doc),
            DocAnchor::root(Transform::from_translation(offset)),
        );
        ctx.app.update();
    }

    assert_eq!(prims(&mut ctx.app), 1, "the document was never rebuilt");
    assert_eq!(
        ctx.app.world().get::<Transform>(doc).map(|t| t.translation),
        Some(AT)
    );
}
