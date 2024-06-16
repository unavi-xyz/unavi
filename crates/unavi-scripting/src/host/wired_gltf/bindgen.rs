use bevy::utils::HashSet;

use self::wired::{
    gltf::node::Transform,
    math::types::{Quat, Vec3},
};

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-gltf",
    world: "host",
    with: {
        "wired:gltf/material/material": Material,
        "wired:gltf/mesh/mesh": Mesh,
        "wired:gltf/node/node": Node,
    }
});

#[derive(Default)]
pub struct Material {
    pub name: String,
}

#[derive(Default)]
pub struct Mesh {
    pub name: String,
}

#[derive(Default)]
pub struct Node {
    pub children: HashSet<u32>,
    pub mesh: Option<u32>,
    pub name: String,
    pub parent: Option<u32>,
    pub transform: Transform,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            rotation: Quat::default(),
            scale: Vec3::splat(1.0),
            translation: Vec3::splat(0.0),
        }
    }
}

impl Vec3 {
    pub fn splat(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
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
