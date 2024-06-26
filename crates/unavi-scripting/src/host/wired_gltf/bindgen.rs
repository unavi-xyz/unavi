use bevy::{
    math::{Quat, Vec3},
    utils::HashSet,
};
use wasm_bridge::component::Resource;
use wasm_bridge_wasi::ResourceTable;

use crate::host::wired_math::{quat::QuatRes, transform::Transform, vec3::Vec3Res};

use self::wired::gltf::material::Color;

wasm_bridge::component::bindgen!({
    path: "../../wired-protocol/spatial/wit/wired-gltf",
    world: "host",
    with: {
        "wired:gltf/material/material": Material,
        "wired:gltf/mesh/mesh": Mesh,
        "wired:gltf/mesh/primitive": Primitive,
        "wired:gltf/node/node": Node,
        "wired:math/types/vec3": crate::host::wired_math::vec3::Vec3Res,
        "wired:math/types/quat": crate::host::wired_math::quat::QuatRes,
        "wired:math/types/transform": crate::host::wired_math::transform::Transform,
    }
});

#[derive(Default)]
pub struct Material {
    pub name: String,
    pub color: Color,
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

#[derive(Default)]
pub struct Mesh {
    pub name: String,
    pub primitives: HashSet<u32>,
}

#[derive(Default)]
pub struct Primitive {
    pub material: Option<u32>,
}

pub struct Node {
    pub children: HashSet<u32>,
    pub mesh: Option<u32>,
    pub name: String,
    pub parent: Option<u32>,
    pub transform: Resource<Transform>,
}

impl Node {
    pub fn try_new(table: &mut ResourceTable) -> wasm_bridge::Result<Self> {
        let rotation = table.push(QuatRes::new(Quat::default()))?;
        let scale = table.push(Vec3Res::new(Vec3::splat(1.0)))?;
        let translation = table.push(Vec3Res::new(Vec3::default()))?;

        let transform = table.push(Transform {
            translation,
            rotation,
            scale,
        })?;

        Ok(Self {
            children: HashSet::default(),
            mesh: None,
            name: String::default(),
            parent: None,
            transform,
        })
    }
}
