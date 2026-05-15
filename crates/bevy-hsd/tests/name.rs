use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, name::NameAttr},
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_name_lifecycle(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    // Add attribute.
    let attr = NameAttr {
        name: "My Node".to_string(),
    };
    attr.attr_reconcile(meta.clone()).expect("reconcile");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&bevy::prelude::Name>();

    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);

    let out = res[0];
    assert_eq!(out.as_str(), attr.name);

    // Remove attribute.
    meta.delete(NameAttr::KEY).expect("delete");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert!(res.is_empty());
}
