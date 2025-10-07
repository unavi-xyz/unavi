use exports::wired::ecs::guest_api::{Guest, GuestScript};
use wired_ecs::prelude::*;

wit_bindgen::generate!({
    generate_all,
    additional_derives: [wired_ecs::Component],
    with: {
        "wired:ecs/types": wired_ecs::types,
    },
});

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script {
    app: App,
}

impl GuestScript for Script {
    fn new() -> Self {
        let mut app = App::default();
        app.add_system(Schedule::Startup, startup);

        Self { app }
    }

    fn exec_system(&self, id: SystemId, data: Vec<ParamData>) {
        self.app.exec_system(id, data);
    }
}

fn startup(commands: Commands) {}

export!(World);
