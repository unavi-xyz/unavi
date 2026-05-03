use crate::wired::scene::{api::self_document, types::Document};

wired_prelude::generate_script!(Script);

struct Script {
    doc: Document,
}

impl GuestScript for Script {
    fn new() -> Self {
        println!("> construct!");
        let doc = self_document();
        Self { doc }
    }

    fn tick(&self) {
        println!("> tick!");

        for _n in self.doc.roots() {
            // let id = n.id();
            // println!("  - node {id}");
        }
    }

    fn render(&self) {}

    fn drop(&self) {}
}
