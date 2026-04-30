mod document;
mod material;
mod mesh;
mod node;

pub mod bindings {
    use super::*;

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.node": node::NodeRes,
            // "wired:scene/types.material": material::HostMaterial,
            // "wired:scene/types.mesh": mesh::HostMesh,
            // "wired:scene/types.document": document::HostDocument,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

pub struct WiredScene {}
