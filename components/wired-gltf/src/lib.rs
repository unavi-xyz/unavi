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

    fn remove_mesh(id: u32) {
        println!("Mesh removed: {}", id);
    }
}

bindings::export!(Component with_types_in bindings);
