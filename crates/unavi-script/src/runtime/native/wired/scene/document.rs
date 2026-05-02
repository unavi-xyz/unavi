use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::wired::scene::{doc::DocRes, material::MaterialRes, mesh::MeshRes, node::NodeRes},
};

use super::bindings::wired::{
    math::types::{Quat, Transform, Vec3},
    scene::types::HostDocument,
};

impl HostDocument for Runtime {
    async fn id(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<u8>> {
        self.backend
            .wired_scene
            .lock()
            .await
            .doc_id(self_.rep())
            .ok_or_else(|| wasmtime::Error::msg("invalid doc"))
    }

    async fn clone(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Resource<DocRes>> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_clone(self_.rep())
            .ok_or_else(|| wasmtime::Error::msg("invalid doc"))?;
        Ok(Resource::new_own(rep))
    }

    async fn roots(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        let reps = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_roots(self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(reps.into_iter().map(Resource::new_own).collect())
    }

    async fn nodes(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        let reps = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_nodes(self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(reps.into_iter().map(Resource::new_own).collect())
    }

    async fn meshes(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<MeshRes>>> {
        let reps = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_meshes(self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(reps.into_iter().map(Resource::new_own).collect())
    }

    async fn materials(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<MaterialRes>>> {
        let reps = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_materials(self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(reps.into_iter().map(Resource::new_own).collect())
    }

    async fn create_node(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<NodeRes>> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_create_node(self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Resource::new_own(rep))
    }

    async fn create_mesh(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<MeshRes>> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_create_mesh(self_.rep())
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Resource::new_own(rep))
    }

    async fn create_material(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<MaterialRes>> {
        let rep = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_create_material(self_.rep())
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(Resource::new_own(rep))
    }

    async fn remove_node(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<NodeRes>,
    ) -> wasmtime::Result<()> {
        self.backend
            .wired_scene
            .lock()
            .await
            .doc_remove_node(value.rep());
        Ok(())
    }

    async fn remove_mesh(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<MeshRes>,
    ) -> wasmtime::Result<()> {
        self.backend
            .wired_scene
            .lock()
            .await
            .doc_remove_mesh(value.rep());
        Ok(())
    }

    async fn remove_material(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<MaterialRes>,
    ) -> wasmtime::Result<()> {
        self.backend
            .wired_scene
            .lock()
            .await
            .doc_remove_material(value.rep());
        Ok(())
    }

    async fn sync(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<bool> {
        Ok(false)
    }

    async fn set_sync(&mut self, _self_: Resource<DocRes>, _value: bool) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn public(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<bool> {
        Ok(false)
    }

    async fn set_public(&mut self, _self_: Resource<DocRes>, _value: bool) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn assets(
        &mut self,
        _self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<(String, Vec<u8>)>> {
        Ok(vec![])
    }

    async fn add_asset(
        &mut self,
        _self_: Resource<DocRes>,
        _name: String,
        _blob_id: Vec<u8>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn remove_asset(
        &mut self,
        _self_: Resource<DocRes>,
        _name: String,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn translation(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec3> {
        let v = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_translation(self_.rep())
            .unwrap_or_default();
        Ok(v.into())
    }

    async fn set_translation(
        &mut self,
        self_: Resource<DocRes>,
        value: Vec3,
    ) -> wasmtime::Result<()> {
        let mut t = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_transform(self_.rep())
            .unwrap_or_default();
        t.translation = value.into();
        self.backend
            .wired_scene
            .lock()
            .await
            .doc_set_transform(self_.rep(), t)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn rotation(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Quat> {
        let q = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_rotation(self_.rep())
            .unwrap_or(bevy::math::Quat::IDENTITY);
        Ok(q.into())
    }

    async fn set_rotation(&mut self, self_: Resource<DocRes>, value: Quat) -> wasmtime::Result<()> {
        let mut t = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_transform(self_.rep())
            .unwrap_or_default();
        t.rotation = value.into();
        self.backend
            .wired_scene
            .lock()
            .await
            .doc_set_transform(self_.rep(), t)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn scale(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec3> {
        let v = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_scale(self_.rep())
            .unwrap_or(bevy::math::Vec3::ONE);
        Ok(v.into())
    }

    async fn set_scale(&mut self, self_: Resource<DocRes>, value: Vec3) -> wasmtime::Result<()> {
        let mut t = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_transform(self_.rep())
            .unwrap_or_default();
        t.scale = value.into();
        self.backend
            .wired_scene
            .lock()
            .await
            .doc_set_transform(self_.rep(), t)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn transform(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Transform> {
        let t = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_transform(self_.rep())
            .unwrap_or_default();
        Ok(t.into())
    }

    async fn set_transform(
        &mut self,
        self_: Resource<DocRes>,
        value: Transform,
    ) -> wasmtime::Result<()> {
        self.backend
            .wired_scene
            .lock()
            .await
            .doc_set_transform(self_.rep(), value.into())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn global_transform(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Transform> {
        let gt = self
            .backend
            .wired_scene
            .lock()
            .await
            .doc_global_transform(self_.rep())
            .unwrap_or_default();
        Ok(gt.into())
    }

    async fn drop(&mut self, rep: Resource<DocRes>) -> wasmtime::Result<()> {
        self.backend.wired_scene.lock().await.docs.remove(rep.rep());
        Ok(())
    }
}
