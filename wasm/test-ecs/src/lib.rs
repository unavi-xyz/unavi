use exports::wired::ecs::guest_api::{Guest, GuestScript};
use wired_ecs::prelude::*;

mod component;
mod constraint;
mod entity;
mod local;
mod query;
mod resource;
mod system_order;

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
        let mut app = App::default();

        component::add(&mut app);
        constraint::add(&mut app);
        entity::add(&mut app);
        local::add(&mut app);
        query::add(&mut app);
        resource::add(&mut app);
        system_order::add(&mut app);

        Self { app }
    }

    fn exec_system(&self, id: SystemId, data: Vec<ParamData>) {
        self.app.exec_system(id, data);
    }
}

export!(World);
