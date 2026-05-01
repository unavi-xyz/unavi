use crate::wired::scene::{api::self_document, types::Document};

wired_prelude::generate_script!(Script);

struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        println!("> construct!");
        let doc = self_document();
        Self {}
    }

    fn tick(&self) {
        println!("> tick!");
    }

    fn render(&self) {}

    fn drop(&self) {}
}
