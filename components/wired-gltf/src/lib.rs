#[allow(warnings)]
mod bindings;

use bindings::exports::wired::gltf::types::{Guest, Mesh};

struct Component;

impl Guest for Component {
    fn meshes() -> Vec<Mesh> {
        println!("Meshes called");

        vec![Mesh {
            id: 0,
            name: "Cube".to_string(),
            extras: None,
        }]
    }

    fn spawn_mesh() -> Mesh {
        println!("Mesh spawned");

        Mesh {
            id: 0,
            name: "Cube".to_string(),
            extras: None,
        }
    }

    fn add_mesh(m: Mesh) {
        println!("Mesh added: {:?}", m);
    }

    fn remove_mesh(m: Mesh) {
        println!("Mesh removed: {:?}", m);
    }
}

bindings::export!(Component with_types_in bindings);
