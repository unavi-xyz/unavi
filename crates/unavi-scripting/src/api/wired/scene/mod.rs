use std::sync::{Arc, RwLock};

use anyhow::Result;
use bevy::{prelude::*, utils::HashMap};
use wasm_bridge::component::{Linker, Resource};

use crate::data::ScriptData;

use self::composition::Composition;

pub mod composition;
pub mod document;
pub mod material;
pub mod mesh;
pub mod nodes;
#[allow(clippy::module_inception)]
pub mod scene;

pub mod bindings {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-scene",
        with: {
            "wired:input": crate::api::wired::input::bindings,
            "wired:math": crate::api::wired::math::bindings,
            "wired:physics": crate::api::wired::physics::bindings,
            "wired:scene/composition/asset-node": super::nodes::base::NodeRes,
            "wired:scene/composition/composition": super::composition::Composition,
            "wired:scene/document/document": super::document::Document,
            "wired:scene/material/material": super::material::MaterialRes,
            "wired:scene/mesh/mesh": super::mesh::MeshRes,
            "wired:scene/mesh/primitive": super::mesh::PrimitiveRes,
            "wired:scene/node/node": super::nodes::base::NodeRes,
            "wired:scene/scene/scene": super::scene::SceneRes,
        }
    });

    pub use wired::scene::*;
}

pub fn add_to_linker(linker: &mut Linker<ScriptData>) -> Result<()> {
    bindings::api::add_to_linker(linker, |s| s)?;
    bindings::composition::add_to_linker(linker, |s| s)?;
    bindings::document::add_to_linker(linker, |s| s)?;
    bindings::material::add_to_linker(linker, |s| s)?;
    bindings::mesh::add_to_linker(linker, |s| s)?;
    bindings::node::add_to_linker(linker, |s| s)?;
    bindings::scene::add_to_linker(linker, |s| s)?;
    Ok(())
}

pub struct WiredScene {
    pub default_material: Handle<StandardMaterial>,
    pub entities: Entities,
    pub root: Resource<Composition>,
}

#[derive(Default)]
pub struct Entities {
    pub compositions: Arc<RwLock<HashMap<u32, Entity>>>,
    pub documents: Arc<RwLock<HashMap<u32, Entity>>>,
    pub materials: Arc<RwLock<HashMap<u32, MaterialState>>>,
    pub nodes: Arc<RwLock<HashMap<u32, Entity>>>,
    pub primitives: Arc<RwLock<HashMap<u32, Handle<Mesh>>>>,
    pub scenes: Arc<RwLock<HashMap<u32, Entity>>>,
}

pub struct MaterialState {
    pub entity: Entity,
    pub handle: Handle<StandardMaterial>,
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

impl bindings::api::Host for ScriptData {
    fn root(&mut self) -> wasm_bridge::Result<Resource<Composition>> {
        let res = self.clone_res(&self.api.wired_scene.as_ref().unwrap().root)?;
        Ok(res)
    }
}
