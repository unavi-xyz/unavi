use bevy::prelude::Name;
use hsd::attributes::name::NameAttr;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_name_lifecycle(mut ctx: TestContext) {
    let root = ctx.create_prim();

    let attr = NameAttr("My Node".to_string());
    ctx.set_attr(root, &attr);

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&Name>();

    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].as_str(), attr.0);

    ctx.remove_attr::<NameAttr>(root);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 0);
}
