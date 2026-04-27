mod document;
mod material;
mod mesh;
mod node;

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.node": super::node::NodeRes,
            // "wired:scene/types.material": super::material::HostMaterial,
            // "wired:scene/types.mesh": super::mesh::HostMesh,
            // "wired:scene/types.document": super::document::HostDocument,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

pub struct WiredSceneRt {}
