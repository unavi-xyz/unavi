use std::cell::RefCell;

use bindings::{
    exports::wired::script::types::{Guest, GuestScript, Script, ScriptBorrow},
    wired::ecs::types::{EcsWorld, Query},
};
use store::Store;

#[allow(warnings)]
mod bindings;

mod store;

#[derive(Clone)]
struct Count {
    increment: usize,
    value: usize,
}

struct ScriptImpl {
    store: RefCell<Store<Count>>,
    query: Query,
}

impl GuestScript for ScriptImpl {}

struct UnaviSystem;

impl Guest for UnaviSystem {
    type Script = ScriptImpl;

    fn init(ecs_world: &EcsWorld) -> Script {
        let component = ecs_world.register_component();
        let query = ecs_world.register_query(&[&component]);

        let mut store = Store::new(component);

        ecs_world.spawn(vec![store.insert_new(Count {
            value: 0,
            increment: 2,
        })]);

        let script = Script::new(ScriptImpl {
            store: store.into(),
            query,
        });

        println!("script init");

        script
    }

    fn update(ecs_world: &EcsWorld, script: ScriptBorrow) {
        let script: &ScriptImpl = script.get();
        println!("script update");

        let mut store = script.store.borrow_mut();

        for (entity, count_comp) in script.query.read() {
            let count = store.get(&count_comp).unwrap();
            println!("count: {}", count.value);

            let mut count = count.clone();
            count.value += count.increment;

            store.insert(&count_comp, count)
        }
    }
}

bindings::export!(UnaviSystem with_types_in bindings);
