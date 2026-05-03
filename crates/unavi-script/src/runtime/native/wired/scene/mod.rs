use wasmtime::component::{HasSelf, Linker, Resource};

use crate::runtime::{
    Runtime,
    shared::wired::scene::{document::DocRes, node::NodeRes},
};

pub mod document;
pub mod material;
pub mod mesh;
pub mod node;

pub mod bindings {
    pub use crate::runtime::shared::wired::scene::{
        document::DocRes, material::MaterialRes, mesh::MeshRes, node::NodeRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-scene",
        with: {
            "wired:scene/types.document": DocRes,
            "wired:scene/types.material": MaterialRes,
            "wired:scene/types.mesh":     MeshRes,
            "wired:scene/types.node":     NodeRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

impl bindings::wired::scene::api::Host for Runtime {
    async fn self_node(&mut self) -> wasmtime::Result<Resource<NodeRes>> {
        let rep = self.backend.wired_scene.lock().await.self_node();
        Ok(Resource::new_own(rep))
    }

    async fn self_document(&mut self) -> wasmtime::Result<Resource<DocRes>> {
        let rep = self.backend.wired_scene.lock().await.self_document();
        Ok(Resource::new_own(rep))
    }

    async fn get_document(&mut self, id: Vec<u8>) -> wasmtime::Result<Option<Resource<DocRes>>> {
        let rep = self.backend.wired_scene.lock().await.get_document(id);
        Ok(rep.map(Resource::new_own))
    }

    async fn create_document(&mut self) -> wasmtime::Result<Resource<DocRes>> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .create_document()
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Resource::new_own(rep))
    }

    async fn remove_document(&mut self, id: Vec<u8>) -> wasmtime::Result<()> {
        self.backend.wired_scene.lock().await.remove_document(id);
        Ok(())
    }

    async fn load_hsd(
        &mut self,
        blob_id: Vec<u8>,
    ) -> wasmtime::Result<Result<Resource<DocRes>, String>> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .load_hsd(blob_id)
            .await
            .map_err(|e| e.to_string());
        Ok(rep.map(Resource::new_own))
    }
}

impl bindings::wired::scene::types::Host for Runtime {}

use bevy::transform::components::{GlobalTransform, Transform};
use bindings::wired::math::types::{Quat as WitQuat, Transform as WitTransform, Vec3 as WitVec3};

impl From<bevy::math::Vec3> for WitVec3 {
    fn from(v: bevy::math::Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<WitVec3> for bevy::math::Vec3 {
    fn from(v: WitVec3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

impl From<bevy::math::Quat> for WitQuat {
    fn from(q: bevy::math::Quat) -> Self {
        Self {
            x: q.x,
            y: q.y,
            z: q.z,
            w: q.w,
        }
    }
}

impl From<WitQuat> for bevy::math::Quat {
    fn from(q: WitQuat) -> Self {
        Self::from_xyzw(q.x, q.y, q.z, q.w)
    }
}

impl From<Transform> for WitTransform {
    fn from(t: Transform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

impl From<WitTransform> for Transform {
    fn from(t: WitTransform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

impl From<GlobalTransform> for WitTransform {
    fn from(gt: GlobalTransform) -> Self {
        let (scale, rotation, translation) = gt.to_scale_rotation_translation();
        Self {
            translation: translation.into(),
            rotation: rotation.into(),
            scale: scale.into(),
        }
    }
}
