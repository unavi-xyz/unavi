use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    PrimMeta,
    attributes::{
        Attribute,
        Attributes,
        attributes_map,
        name::NameAttr,
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
fn test_name_lifecycle(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    let attr = NameAttr("My Node".to_string());
    let prim = PrimMeta {
        attributes: Some(Attributes {
            name: Some(attr.clone()),
            ..Default::default()
        }),
        ..Default::default()
    };
    prim.reconcile(RootReconciler::new(meta.clone()))
        .expect("reconcile");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&bevy::prelude::Name>();

    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].as_str(), attr.0);

    let attrs = attributes_map(&meta).expect("attributes map");
    attrs.delete(NameAttr::KEY).expect("delete");

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert!(res.is_empty());
}
