wit_bindgen::generate!({
    world: "button",
    exports: {
        world: Button,
    },
});

pub struct Button;

impl Guest for Button {
    fn press() {
        // log(LogLevel::Info, "Button press");
    }
}
