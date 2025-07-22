use exports::wired::script::types::{Guest, GuestScript};
use wired::scene::{mesh::Mesh, node::Node, scene::Scene};

wit_bindgen::generate!({ generate_all });

struct World;

impl Guest for World {
    type Script = Script;
}

#[allow(dead_code)]
struct Script {
    scene: Scene,
}

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        let node = Node::new();
        scene.add_node(&node);

        let mesh = Mesh::new();
        node.set_mesh(Some(&mesh));

        Self { scene }
    }

    fn update(&self, _delta: f32) {}

    fn render(&self, _delta: f32) {}
}

export!(World);
