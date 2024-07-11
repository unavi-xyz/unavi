use anyhow::Result;
use wasm_bridge::component::Linker;
use wired::{
    math::types::{Quat, Vec3},
    scene::{material::Color, node::Transform},
};

use crate::state::StoreState;

pub mod gltf;
pub mod glxf;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-scene",
    with: {
        "wired:input/handler/input-handler": super::wired_input::input_handler::InputHandler,
        "wired:physics/types/collider": super::wired_physics::collider::Collider,
        "wired:physics/types/rigid-body": super::wired_physics::rigid_body::RigidBody,
        "wired:scene/gltf/gltf": gltf::document::GltfDocument,
        "wired:scene/glxf/asset-gltf": glxf::asset_gltf::GltfAssetRes,
        "wired:scene/glxf/asset-glxf": glxf::asset_glxf::GlxfAssetRes,
        "wired:scene/glxf/glxf": glxf::document::GlxfDocument,
        "wired:scene/glxf/glxf-node": glxf::node::GlxfNodeRes,
        "wired:scene/glxf/glxf-scene": glxf::scene::GlxfSceneRes,
        "wired:scene/material/material": gltf::material::MaterialRes,
        "wired:scene/mesh/mesh": gltf::mesh::MeshRes,
        "wired:scene/mesh/primitive": gltf::mesh::PrimitiveRes,
        "wired:scene/node/node": gltf::node::NodeRes,
        "wired:scene/scene/scene": gltf::scene::SceneRes,
    }
});

pub fn add_to_linker(linker: &mut Linker<StoreState>) -> Result<()> {
    wired::scene::gltf::add_to_linker(linker, |s| s)?;
    wired::scene::glxf::add_to_linker(linker, |s| s)?;
    wired::scene::material::add_to_linker(linker, |s| s)?;
    wired::scene::mesh::add_to_linker(linker, |s| s)?;
    wired::scene::node::add_to_linker(linker, |s| s)?;
    wired::scene::scene::add_to_linker(linker, |s| s)?;
    Ok(())
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            rotation: Quat::default(),
            scale: Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            translation: Vec3::default(),
        }
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}
