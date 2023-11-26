wit_bindgen::generate!({
    world: "button",

    exports: {
        world: MyButton,
    },
});

struct MyButton;

impl Guest for MyButton {
    fn run() {
        print("Hello, world!");
    }
}
