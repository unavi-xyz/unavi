use crate::wired::scene::{
    api::self_document,
    types::{Collider, RigidBodyKind},
};
use wired_prelude::wired_math::types::{Quat, Transform, Vec3};

use crate::check;

pub fn test_node() {
    let doc = self_document();
    let node = doc.create_node();
    let mesh = doc.create_mesh();
    let mat = doc.create_material();
    let child = doc.create_node();

    check("node id non-empty", !node.id().is_empty(), true);

    node.set_name(Some("test-node"));
    check("node name", node.name().as_deref(), Some("test-node"));
    node.set_name(None);
    check("node name cleared", node.name(), None::<String>);

    let t = Vec3::new(1.0, 2.0, 3.0);
    node.set_translation(t);
    check("node translation", node.translation(), t);

    let q = Quat::new(0.0, 0.0, 0.0, 1.0);
    node.set_rotation(q);
    check("node rotation", node.rotation(), q);

    let s = Vec3::new(2.0, 2.0, 2.0);
    node.set_scale(s);
    check("node scale", node.scale(), s);

    let tr = Transform {
        translation: Vec3::new(4.0, 5.0, 6.0),
        rotation: Quat::new(0.0, 0.0, 0.0, 1.0),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    node.set_transform(tr);
    check("node transform", node.transform(), tr);

    node.add_child(&child);
    check("add_child count", node.children().len(), 1);
    check("child parent is some", child.parent().is_some(), true);
    node.remove_child(&child);
    check("remove_child count", node.children().len(), 0);

    node.set_mesh(Some(&mesh));
    check("node mesh is some", node.mesh().is_some(), true);
    node.set_mesh(None);
    check("node mesh cleared", node.mesh().is_none(), true);

    node.set_material(Some(&mat));
    check("node material is some", node.material().is_some(), true);
    node.set_material(None);
    check("node material cleared", node.material().is_none(), true);

    node.set_collider(Some(&Collider::Sphere(0.5)));
    check("node collider is some", node.collider().is_some(), true);

    node.set_rigid_body(Some(RigidBodyKind::Dynamic));
    check("node rigid_body is some", node.rigid_body().is_some(), true);

    check("node sync default", node.sync(), false);
    node.set_sync(true);
    check("node sync set", node.sync(), true);
    node.set_sync(false);
}
