wired_prelude::generate_script!(Script);

mod document;
mod material;
mod mesh;
mod node;

struct Script;

fn check<T: PartialEq + std::fmt::Debug>(label: &str, got: T, expected: T) {
    if got == expected {
        println!("pass: {label}");
    } else {
        eprintln!("FAIL {label}: got {got:?}, expected {expected:?}");
    }
}

impl GuestScript for Script {
    fn new() -> Self {
        document::test_document();
        node::test_node();
        material::test_material();
        mesh::test_mesh();
        println!("tests complete");
        Self
    }

    fn tick(&self) {}

    fn render(&self) {}

    fn drop(&self) {}
}
