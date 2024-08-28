use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::scene::api::{Root, Scene},
    wired::{
        log::api::{log, LogLevel},
        player::api::{local_player, Skeleton},
    },
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;

struct Script {
    skeleton: Skeleton,
}

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        Root::add_scene(&scene);

        Script {
            skeleton: local_player().skeleton(),
        }
    }

    fn update(&self, _delta: f32) {
        log(
            LogLevel::Info,
            &format!(
                "left_lower_arm: {:?}",
                self.skeleton.left_lower_arm.transform().translation
            ),
        );
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
