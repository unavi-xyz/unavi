wit_bindgen::generate!({
    world: "button",
    exports: {
        world: UnaviButton,
    },
});

struct UnaviButton;

impl Guest for UnaviButton {
    fn press(state: HoverState) {
        println!("Button pressed with state: {:?}", state);
    }
}
