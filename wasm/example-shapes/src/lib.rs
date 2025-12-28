use bindings::exports::wired::ecs::guest_api::{Guest, GuestScript};
use wired_ecs::prelude::*;

wired_ecs::generate_bindgen!();

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script {
    app: App,
}

impl GuestScript for Script {
    fn new() -> Self {
        let app = App::default();
        Self { app }
    }

    fn exec_system(&self, id: SystemId, data: Vec<ParamData>) {
        self.app.exec_system(id, data);
    }
}

bindings::export!(World);
