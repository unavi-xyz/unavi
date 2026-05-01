use crate::wired::scene::api::self_document;

wired_prelude::generate_script!(Script);

struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        println!("> construct!");
        let _doc = self_document();
        Self {}
    }

    fn tick(&self) {
        println!("> tick!");
    }

    fn render(&self) {}

    fn drop(&self) {}
}
