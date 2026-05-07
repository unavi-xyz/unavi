use crate::wired::scene::{api::self_document, types::Document};

wired_prelude::generate_script!(Script);

struct Script {
    doc: Document,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        println!("> construct!");
        let doc = self_document();
        Self { doc }
    }

    fn tick(&mut self) {
        println!("> tick!");

        for _n in self.doc.roots() {
            // let id = n.id();
            // println!("  - node {id}");
        }
    }
}
