use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::vscreen::screen::{Screen, ScreenShape},
    wired::{
        player::api::{local_player, Skeleton},
        scene::gltf::Node,
    },
};

#[allow(warnings)]
mod bindings;
mod clock;
mod wired_input_impls;
mod wired_math_impls;
mod wired_scene_impls;

const ACTIVE_Y: f32 = 1.2;

struct Script {
    root: Node,
    screen: Screen,
    skeleton: Skeleton,
}

impl GuestScript for Script {
    fn new() -> Self {
        let screen = Screen::new(ScreenShape::Circle(0.03));

        let player = local_player();
        let skeleton = player.skeleton();

        skeleton.left_hand.add_child(&screen.root().root());

        Script {
            root: player.root(),
            screen,
            skeleton,
        }
    }

    fn update(&self, delta: f32) {
        let relative_y = self.skeleton.left_hand.global_transform().translation.y
            - self.root.global_transform().translation.y;

        self.screen.set_visible(relative_y > ACTIVE_Y);

        self.screen.update(delta);
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
