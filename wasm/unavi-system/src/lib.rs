use std::f32::consts::{FRAC_PI_2, PI};

use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    wired::{
        math::types::{Quat, Transform, Vec3},
        player::api::{local_player, Skeleton},
        scene::gltf::Node,
    },
};
use clock::Clock;

#[allow(warnings)]
mod bindings;
mod clock;
mod wired_input_impls;
mod wired_math_impls;
mod wired_scene_impls;

const ACTIVE_Y: f32 = 1.2;
const ARM_RADIUS: f32 = 0.03;
const SCREEN_RADIUS: f32 = 0.025;

struct Script {
    clock: Clock,
    root: Node,
    skeleton: Skeleton,
}

impl GuestScript for Script {
    fn new() -> Self {
        let clock = Clock::default();
        clock.screen.root().root().set_transform(Transform {
            translation: Vec3::new(SCREEN_RADIUS * 2.0, ARM_RADIUS, 0.0),
            rotation: Quat::from_rotation_x(-FRAC_PI_2) * Quat::from_rotation_z(PI),
            ..Default::default()
        });

        let player = local_player();
        let skeleton = player.skeleton();

        skeleton.left_hand.add_child(&clock.screen.root().root());

        Script {
            clock,
            root: player.root(),
            skeleton,
        }
    }

    fn update(&self, delta: f32) {
        self.clock.update(delta);

        let relative_y = self.skeleton.left_hand.global_transform().translation.y
            - self.root.global_transform().translation.y;
        self.clock.screen.set_visible(relative_y > ACTIVE_Y);

        self.clock.screen.update(delta);
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
