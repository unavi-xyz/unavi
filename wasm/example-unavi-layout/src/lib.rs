use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        layout::container::{Alignment, Container},
        scene::api::{Root, Scene},
        shapes::api::Cuboid,
    },
    wired::{
        math::types::{Transform, Vec3},
        physics::types::{Collider, Shape},
        scene::node::Node,
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        // Container alignments
        let root = Node::new();
        root.set_transform(Transform::from_translation(Vec3::new(4.0, 0.0, 0.0)));

        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    let size = Vec3::splat(1.0);
                    let parent = Container::new(size);
                    root.add_child(&parent.root());

                    let collider = Collider::new(Shape::Cuboid(size));
                    parent.root().set_collider(Some(&collider));
                    parent
                        .root()
                        .set_transform(Transform::from_translation(Vec3::new(
                            x as f32, y as f32, z as f32,
                        )));

                    parent.set_align_x(get_alignment(x));
                    parent.set_align_y(get_alignment(y));
                    parent.set_align_z(get_alignment(z));

                    let child_size = Vec3::splat(0.2);
                    let child = Container::new(child_size);
                    parent.add_child(&child);

                    let mesh = Cuboid::new(child_size).to_mesh();
                    child.inner().set_mesh(Some(&mesh));
                }
            }
        }

        Root::add_scene(&scene);

        Script {}
    }

    fn update(&self, _delta: f32) {}
}

fn get_alignment(num: usize) -> Alignment {
    match num {
        0 => Alignment::Start,
        1 => Alignment::Center,
        2.. => Alignment::End,
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
