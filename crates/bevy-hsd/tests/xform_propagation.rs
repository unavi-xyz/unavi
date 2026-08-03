use bevy::prelude::*;
use bevy_hsd::HsdPrimIndex;
use hsd::{
    HSD_CONTAINER_ID,
    PrimMeta,
    attributes::{
        Attribute,
        Attributes,
        name::NameAttr,
        xform::XformAttr,
    },
};
use loro::LoroMap;
use loro_surgeon::{
    Reconcile,
    reconcile::RootReconciler,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

fn write_attr<A: Attribute>(meta: &LoroMap, attr: &A) {
    let attrs = meta.ensure_mergeable_map("attributes").expect("attrs");
    attr.attr_reconcile(attrs).expect("reconcile");
}

const fn xform(t: [f32; 3]) -> XformAttr {
    XformAttr {
        rotation:    [0.0, 0.0, 0.0, 1.0],
        scale:       [1.0, 1.0, 1.0],
        translation: t,
    }
}

/// A parent prim reconciled with `xform: None` (its `Attributes` has other
/// fields but no xform — as the gate's root prim, which only carries name +
/// script) still emits a null xform attr event. That must not strip the prim's
/// `require`d `Transform`, or transform propagation to its children collapses
/// them onto the parent's origin.
#[traced_test]
#[rstest]
fn root_without_xform_keeps_child_transforms(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);

    let root = tree.create(None).expect("root");
    let root_meta = tree.get_meta(root).expect("root meta");
    PrimMeta {
        attributes: Some(Attributes {
            name: Some(NameAttr("gate".to_string())),
            ..Default::default()
        }),
        ..Default::default()
    }
    .reconcile(RootReconciler::new(root_meta))
    .expect("reconcile root");

    let translations = [[-1.0f32, 5.0, 0.0], [1.0, 5.0, 0.0], [0.0, 10.0, 0.0]];
    let mut children = Vec::new();
    for t in translations {
        let child = tree.create(Some(root)).expect("child");
        let meta = tree.get_meta(child).expect("meta");
        write_attr(&meta, &xform(t));
        children.push((child, t));
    }

    ctx.doc.commit();
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
        "root prim lost its required Transform after a null xform attr event"
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
