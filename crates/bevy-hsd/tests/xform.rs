use bevy::prelude::*;
use bevy_hsd::Hsd;
use hsd::attributes::xform::XformAttr;
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_xform_lifecycle(mut ctx: TestContext) {
    let root = ctx.create_prim();

    let attr = XformAttr {
        rotation:    [0.4, 0.5, 0.6, 0.9],
        scale:       [0.9, 0.8, 0.7],
        translation: [1.0, 2.0, 3.0],
    };
    ctx.set_attr(root, &attr);

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query_filtered::<&Transform, Without<Hsd>>();

    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);

    let out = res[0];
    assert!(
        out.rotation
            .abs_diff_eq(Quat::from_array(attr.rotation).normalize(), f32::EPSILON),
        "a rotation reaches Transform normalized, whatever the document stores"
    );
    assert!(
        out.scale
            .abs_diff_eq(Vec3::from_array(attr.scale), f32::EPSILON)
    );
    assert!(
        out.translation
            .abs_diff_eq(Vec3::from_array(attr.translation), f32::EPSILON)
    );

    ctx.remove_attr::<XformAttr>(root);
    ctx.app.update();

    // `Prim` requires `Transform`, so removal resets to identity rather than
    // stripping the component, which would break propagation to any children.
    let world = ctx.app.world_mut();
    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);
    assert_eq!(*res[0], Transform::IDENTITY);
}
