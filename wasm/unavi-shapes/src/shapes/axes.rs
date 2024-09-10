use std::{cell::Cell, f32::consts::FRAC_PI_2, sync::LazyLock};

use crate::bindings::{
    exports::unavi::shapes::api::{GuestAxes, GuestCylinder},
    wired::{
        math::types::{Quat, Transform, Vec3},
        scene::{
            material::{Color, Material},
            node::Node,
        },
    },
};

use super::cylinder::Cylinder;

static MATERIALS: LazyLock<(Material, Material, Material)> = LazyLock::new(|| {
    let red = Material::new();
    red.set_color(Color {
        r: 1.0,
        g: 0.1,
        b: 0.1,
        a: 0.9,
    });
    let green = Material::new();
    green.set_color(Color {
        r: 0.1,
        g: 1.0,
        b: 0.1,
        a: 0.9,
    });
    let blue = Material::new();
    blue.set_color(Color {
        r: 0.1,
        g: 0.1,
        b: 1.0,
        a: 0.9,
    });
    (red, green, blue)
});

pub struct Axes {
    size: Cell<f32>,
}

impl GuestAxes for Axes {
    fn new() -> Self {
        Self {
            size: Cell::new(1.0),
        }
    }

    fn size(&self) -> f32 {
        self.size.get()
    }
    fn set_size(&self, value: f32) {
        self.size.set(value);
    }

    fn to_node(&self) -> crate::bindings::exports::unavi::shapes::api::Node {
        let size = self.size.get();

        let shape = Cylinder::new(0.01, size);
        shape.set_resolution(8);

        let root = Node::new();

        let x = shape.to_node();
        root.add_child(&x);
        x.mesh().unwrap().list_primitives()[0].set_material(Some(&MATERIALS.0));
        x.set_transform(Transform {
            translation: Vec3::new(size / 2.0, 0.0, 0.0),
            rotation: Quat::from_rotation_z(FRAC_PI_2),
            ..Default::default()
        });

        let y = shape.to_node();
        root.add_child(&y);
        y.mesh().unwrap().list_primitives()[0].set_material(Some(&MATERIALS.1));
        y.set_transform(Transform::from_translation(Vec3::new(0.0, size / 2.0, 0.0)));

        let z = shape.to_node();
        root.add_child(&z);
        z.mesh().unwrap().list_primitives()[0].set_material(Some(&MATERIALS.2));
        z.set_transform(Transform {
            translation: Vec3::new(0.0, 0.0, size / 2.0),
            rotation: Quat::from_rotation_x(FRAC_PI_2),
            ..Default::default()
        });

        root
    }
}
