use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::wired::scene::{doc::DocRes, node::NodeRes},
};

use super::{
    bindings::wired::{
        math::types::{Quat, Transform, Vec3},
        scene::types::HostDocument,
    },
    material::MaterialRes,
    mesh::MeshRes,
};

impl HostDocument for Runtime {
    async fn id(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<Vec<u8>> {
        todo!()
    }

    async fn clone(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<Resource<DocRes>> {
        todo!()
    }

    async fn roots(
        &mut self,
        _self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        Ok(vec![])
    }

    async fn nodes(
        &mut self,
        _self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        Ok(vec![])
    }

    async fn meshes(
        &mut self,
        _self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<MeshRes>>> {
        Ok(vec![])
    }

    async fn materials(
        &mut self,
        _self_: Resource<DocRes>,
    ) -> wasmtime::Result<Vec<Resource<MaterialRes>>> {
        Ok(vec![])
    }

    async fn create_node(
        &mut self,
        _self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<NodeRes>> {
        todo!()
    }

    async fn create_mesh(
        &mut self,
        _self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<MeshRes>> {
        let res = self.native.table.push(MeshRes)?;
        Ok(res)
    }

    async fn create_material(
        &mut self,
        _self_: Resource<DocRes>,
    ) -> wasmtime::Result<Resource<MaterialRes>> {
        let res = self.native.table.push(MaterialRes)?;
        Ok(res)
    }

    async fn remove_node(
        &mut self,
        _self_: Resource<DocRes>,
        _value: Resource<NodeRes>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn remove_mesh(
        &mut self,
        _self_: Resource<DocRes>,
        _value: Resource<MeshRes>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn remove_material(
        &mut self,
        _self_: Resource<DocRes>,
        _value: Resource<MaterialRes>,
    ) -> wasmtime::Result<()> {
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

    async fn translation(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<Vec3> {
        todo!()
    }

    async fn set_translation(
        &mut self,
        _self_: Resource<DocRes>,
        _value: Vec3,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn rotation(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<Quat> {
        todo!()
    }

    async fn set_rotation(
        &mut self,
        _self_: Resource<DocRes>,
        _value: Quat,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn scale(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<Vec3> {
        todo!()
    }

    async fn set_scale(&mut self, _self_: Resource<DocRes>, _value: Vec3) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn transform(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<Transform> {
        todo!()
    }

    async fn set_transform(
        &mut self,
        _self_: Resource<DocRes>,
        _value: Transform,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn global_transform(&mut self, _self_: Resource<DocRes>) -> wasmtime::Result<Transform> {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<DocRes>) -> wasmtime::Result<()> {
        self.native.table.delete(rep)?;
        Ok(())
    }
}
