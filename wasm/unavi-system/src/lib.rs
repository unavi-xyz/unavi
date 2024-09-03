use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::vscreen::screen::{Module, Screen},
    wired::{
        log::api::{log, LogLevel},
        player::api::{local_player, Skeleton},
    },
};

#[allow(warnings)]
mod bindings;
mod clock;
mod wired_math_impls;
mod wired_scene_impls;

struct Script {
    screen: Screen,
    skeleton: Skeleton,
}

impl GuestScript for Script {
    fn new() -> Self {
        let screen = Screen::new();

        let clock = Module::new();
        screen.set_central_module(Some(&clock));

        let skeleton = local_player().skeleton();

        Script { screen, skeleton }
    }

    fn update(&self, delta: f32) {
        let lower_arm_y = self.skeleton.left_hand.transform().translation.y;
        log(LogLevel::Info, &format!("y: {}", lower_arm_y));

        self.screen.update(delta);
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
