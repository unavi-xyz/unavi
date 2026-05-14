use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, xform::Xform},
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_xform_spawn(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    let xform = Xform {
        translation: vec![1.0, 2.0, 3.0],
        rotation: vec![0.0, 0.0, 0.0, 1.0],
        scale: vec![0.0; 3],
    };
    xform.attr_reconcile(meta).expect("reconcile");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let xforms = world
        .query::<&Transform>()
        .query(world)
        .into_iter()
        .collect::<Vec<_>>();

    assert_eq!(xforms.len(), 1);

    let out = xforms[0];
    assert_eq!(out.translation.to_array().to_vec(), xform.translation);
    assert_eq!(out.rotation.to_array().to_vec(), xform.rotation);
    assert_eq!(out.scale.to_array().to_vec(), xform.scale);
}
