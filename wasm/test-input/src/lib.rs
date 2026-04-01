use std::cell::Cell;

use crate::wired::{
    input::{api::register_input_listener, types::InputListener},
    scene::{context::self_document, types::Collider},
};

wired_prelude::generate_script!(Script);

struct Script {
    listener: InputListener,
    done: Cell<bool>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();
        let node = doc.create_node();
        node.set_collider(Some(&Collider::Sphere(0.5)));
        let listener = register_input_listener(&node);
        Self {
            listener,
            done: Cell::new(false),
        }
    }

    fn tick(&self) {
        if self.done.get() {
            return;
        }
        if self.listener.poll().is_none() {
            println!("pass: input poll returns none");
        } else {
            eprintln!("FAIL input: unexpected event");
        }
        println!("tests complete");
        self.done.set(true);
    }

    fn render(&self) {}

    fn drop(&self) {}
}
