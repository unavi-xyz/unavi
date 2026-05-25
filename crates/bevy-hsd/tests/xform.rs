use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    PrimMeta,
    attributes::{
        Attribute,
        Attributes,
        attributes_map,
        xform::XformAttr,
    },
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
fn test_xform_lifecycle(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    let attr = XformAttr {
        rotation:    [0.4, 0.5, 0.6, 0.9],
        scale:       [0.9, 0.8, 0.7],
        translation: [1.0, 2.0, 3.0],
    };
    let prim = PrimMeta {
        attributes: Some(Attributes {
            xform: Some(attr.clone()),
            ..Default::default()
        }),
        ..Default::default()
    };
    prim.reconcile(RootReconciler::new(meta.clone()))
        .expect("reconcile");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&Transform>();

    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);

    let out = res[0];
    assert_eq!(out.rotation.to_array().to_vec(), attr.rotation);
    assert_eq!(out.scale.to_array().to_vec(), attr.scale);
    assert_eq!(out.translation.to_array().to_vec(), attr.translation);

    let attrs = attributes_map(&meta).expect("attributes map");
    attrs.delete(XformAttr::KEY).expect("delete");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert!(res.is_empty());
}
