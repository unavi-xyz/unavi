wit_bindgen::generate!({
    world: "input",
    exports: {
        world: UnaviInput,
    },
});

struct UnaviInput;

impl Guest for UnaviInput {}
