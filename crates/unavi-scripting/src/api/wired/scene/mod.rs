use anyhow::Result;
use wasm_bridge::component::Linker;

use crate::state::StoreState;

pub mod gltf;
pub mod glxf;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-scene",
        with: {
            "wired:input": crate::api::wired::input::bindings,
            "wired:math": crate::api::wired::math::bindings,
            "wired:physics": crate::api::wired::physics::bindings,
            "wired:scene/gltf/gltf": super::gltf::document::GltfDocument,
            "wired:scene/glxf/asset-gltf": super::glxf::asset_gltf::GltfAssetRes,
            "wired:scene/glxf/asset-glxf": super::glxf::asset_glxf::GlxfAssetRes,
            "wired:scene/glxf/glxf": super::glxf::document::GlxfDocument,
            "wired:scene/glxf/glxf-node": super::glxf::node::GlxfNodeRes,
            "wired:scene/glxf/glxf-scene": super::glxf::scene::GlxfSceneRes,
            "wired:scene/material/material": super::gltf::material::MaterialRes,
            "wired:scene/mesh/mesh": super::gltf::mesh::MeshRes,
            "wired:scene/mesh/primitive": super::gltf::mesh::PrimitiveRes,
            "wired:scene/node/node": super::gltf::node::NodeRes,
            "wired:scene/scene/scene": super::gltf::scene::SceneRes,
        }
    });

    pub use wired::scene::*;
}

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> Result<()> {
    bindings::gltf::add_to_linker(linker, |s| s)?;
    bindings::glxf::add_to_linker(linker, |s| s)?;
    bindings::material::add_to_linker(linker, |s| s)?;
    bindings::mesh::add_to_linker(linker, |s| s)?;
    bindings::node::add_to_linker(linker, |s| s)?;
    bindings::scene::add_to_linker(linker, |s| s)?;
    Ok(())
}

impl Default for bindings::material::Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}