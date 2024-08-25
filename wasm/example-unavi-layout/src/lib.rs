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
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

const SPACING: f32 = 1.0;

struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    let size = Vec3::splat(1.0);

                    let container = Container::new(size);
                    let root = container.root();
                    scene.add_node(&root);

                    let collider = Collider::new(Shape::Cuboid(size));
                    root.set_collider(Some(&collider));

                    root.set_transform(Transform::from_translation(Vec3::new(
                        x as f32 * SPACING,
                        y as f32 * SPACING,
                        z as f32 * SPACING,
                    )));

                    container.set_align_x(get_alignment(x));
                    container.set_align_y(get_alignment(y));
                    container.set_align_z(get_alignment(z));

                    let mesh = Cuboid::new(Vec3::splat(0.2)).to_node();
                    container.inner().add_child(&mesh);
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
