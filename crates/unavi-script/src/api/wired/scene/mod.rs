//! # `wired:scene` API
//!
//! Each script is attached to an HSD document.
//! These APIs interact directly with that underlying Loro document.
//!
//! This allows full Read / Write support outside of the Bevy ECS.

use std::sync::Arc;

use loro::LoroDoc;
use wasmtime_wasi::ResourceTable;

mod material;
mod mesh;
mod node;

mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.node": super::node::HostNode,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

pub struct WiredSceneRt {
    pub table: ResourceTable,
    pub doc: Arc<LoroDoc>,
}

// impl bindings::wired::scene::types::Host for WiredSceneRt {}
