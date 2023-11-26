wit_bindgen::generate!({
    world: "interaction",
    exports: {
        world: UnaviInteraction,
    },
});

struct UnaviInteraction;

impl Guest for UnaviInteraction {
    fn touchup() {}
    fn touchdown() {}
}
