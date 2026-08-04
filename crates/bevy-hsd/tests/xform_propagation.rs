use bevy::prelude::*;
use bevy_hsd::HsdPrimIndex;
use hsd::attributes::{
    name::NameAttr,
    xform::XformAttr,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

const fn xform(t: [f32; 3]) -> XformAttr {
    XformAttr {
        rotation:    [0.0, 0.0, 0.0, 1.0],
        scale:       [1.0, 1.0, 1.0],
        translation: t,
    }
}

/// A parent prim carrying no xform — as the gate's root prim, which only has a
/// name and a script — must keep its `require`d `Transform`, or transform
/// propagation collapses its children onto the parent's origin.
#[traced_test]
#[rstest]
fn root_without_xform_keeps_child_transforms(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(root, &NameAttr("gate".to_string()));

    let translations = [[-1.0f32, 5.0, 0.0], [1.0, 5.0, 0.0], [0.0, 10.0, 0.0]];
    let mut children = Vec::new();
    for t in translations {
        let child = ctx.create_child(root);
        ctx.set_attr(child, &xform(t));
        children.push((child, t));
    }

    // Removing the attribute must reset the transform, never remove it.
    ctx.remove_attr::<XformAttr>(root);

    ctx.app.update();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let index = world
        .query::<&HsdPrimIndex>()
        .iter(world)
        .find(|i| !i.0.is_empty())
        .expect("index");
    let root_ent = *index.0.get(&root).expect("root indexed");
    let ents = children
        .iter()
        .map(|(child, expected)| (*index.0.get(child).expect("child indexed"), *expected))
        .collect::<Vec<_>>();

    assert!(
        world.entity(root_ent).get::<Transform>().is_some(),
        "root prim lost its required Transform after a removed xform attr"
    );

    for (ent, expected) in ents {
        let gt = world
            .entity(ent)
            .get::<GlobalTransform>()
            .expect("child global transform");
        assert!(
            (gt.translation() - Vec3::from_array(expected)).length() < 1.0e-5,
            "child global translation collapsed (parent propagation broken): got {:?}, want {expected:?}",
            gt.translation(),
        );
    }
}
