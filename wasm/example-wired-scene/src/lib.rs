use std::cell::RefCell;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::shapes::api::Cuboid,
    wired::{
        math::types::Vec3,
        scene::{
            api::root,
            composition::{Asset, AssetNode},
            document::Document,
            material::{Color, Material},
            node::Node,
            scene::Scene,
        },
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    color_delta: RefCell<Color>,
    material: Material,
    node: Node,
}

impl GuestScript for Script {
    fn new() -> Self {
        let document = Document::new();

        let scene = Scene::new();
        document.add_scene(&scene);

        let node = Node::new();
        scene.add_node(&node);

        let mesh = Cuboid::new(Vec3::splat(1.0)).to_mesh();
        node.set_mesh(Some(&mesh));

        let material = Material::new();

        for primitive in mesh.list_primitives() {
            primitive.set_material(Some(&material));
        }

        let asset_node = AssetNode::new();
        asset_node.set_asset(Some(Asset::Document(document)));
        root().add_node(&asset_node);

        Script {
            color_delta: RefCell::new(Color {
                r: 0.01,
                g: 0.002,
                b: -0.003,
                a: 0.0,
            }),
            material,
            node,
        }
    }

    fn update(&self, delta: f32) {
        let mut color = self.material.color();
        let mut color_delta = self.color_delta.borrow_mut();

        color.r += color_delta.r;
        color.g += color_delta.g;
        color.b += color_delta.b;
        color.a += color_delta.a;

        color_delta.r = cycle(color_delta.r, color.r);
        color_delta.g = cycle(color_delta.g, color.g);
        color_delta.b = cycle(color_delta.b, color.b);
        color_delta.a = cycle(color_delta.a, color.a);

        self.material.set_color(color);

        let mut transform = self.node.transform();

        let mut quat = glam::Quat::from_xyzw(
            transform.rotation.x,
            transform.rotation.y,
            transform.rotation.z,
            transform.rotation.w,
        );

        quat *= glam::Quat::from_rotation_y(delta);

        transform.rotation.x = quat.x;
        transform.rotation.y = quat.y;
        transform.rotation.z = quat.z;
        transform.rotation.w = quat.w;

        self.node.set_transform(transform);
    }
}

fn cycle(delta: f32, value: f32) -> f32 {
    if value >= 1.0 || value <= 0.0 {
        delta * -1.0
    } else {
        delta
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
