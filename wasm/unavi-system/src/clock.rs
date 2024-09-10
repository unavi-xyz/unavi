use std::f32::consts::{FRAC_PI_2, PI};

use crate::bindings::{
    unavi::{
        ui::text::Text,
        vscreen::screen::{Screen, ScreenShape},
    },
    wired::{
        math::types::{Quat, Transform, Vec3},
        scene::gltf::Node,
    },
};

pub struct Clock {
    pub screen: Screen,
    text: Text,
}

impl Default for Clock {
    fn default() -> Self {
        let screen = Screen::new(ScreenShape::Circle(0.3));

        let text = Text::new("menu");
        text.set_font_size(0.02);

        let text_node = Node::new();
        text_node.set_mesh(Some(&text.mesh()));
        text_node.set_transform(Transform {
            translation: Vec3::new(0.02, 0.002, 0.0),
            rotation: Quat::from_rotation_y(PI) * Quat::from_rotation_x(-FRAC_PI_2),
            ..Default::default()
        });

        screen.root().root().add_child(&text_node);

        Self { screen, text }
    }
}
