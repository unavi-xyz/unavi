use std::collections::HashMap;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript, Script, ScriptBorrow},
    wired::ecs::types::{Component, EcsWorld},
};

#[allow(warnings)]
mod bindings;

struct Count {
    increment: usize,
    value: usize,
}

struct ScriptImpl {
    component: Component,
    data: HashMap<usize, Count>,
}

impl GuestScript for ScriptImpl {}

struct UnaviSystem;

impl Guest for UnaviSystem {
    type Script = ScriptImpl;

    fn init(ecs_world: &EcsWorld) -> Script {
        let component = ecs_world.register_component();

        let script = Script::new(ScriptImpl {
            component,
            data: HashMap::new(),
        });

        println!("script init");

        script
    }

    fn update(ecs_world: &EcsWorld, script: ScriptBorrow) {
        let script: &ScriptImpl = script.get();

        println!("script update");

        // let mut count = script.count.borrow_mut();
        // println!("update count: {}", count);
        // *count += 1;
    }
}

bindings::export!(UnaviSystem with_types_in bindings);
