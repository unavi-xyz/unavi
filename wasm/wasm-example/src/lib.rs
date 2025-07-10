wit_bindgen::generate!({});

struct MyHost;

impl Guest for MyHost {
    fn run() {
        println!("Hello, world!");
    }
}

export!(MyHost);
