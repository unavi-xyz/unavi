use exports::wired::ecs::guest_api::{Guest, GuestScript};
use wired_ecs::{
    App,
    types::{ParamData, Schedule, SystemId},
};

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
        app.add_system(Schedule::Startup, on_startup)
            .add_system(Schedule::Update, on_update);
        Self { app }
    }

    fn exec_system(&self, id: SystemId, data: Vec<ParamData>) {
        self.app.exec_system(id, data);
    }
}

fn on_startup() {
    println!("hello from startup");
}

fn on_update() {
    loop {
        println!("hello from update");
    }
}

export!(World);
