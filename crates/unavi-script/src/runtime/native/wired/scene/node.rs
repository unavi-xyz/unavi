use wasmtime::component::Resource;

use crate::runtime::{Runtime, shared::wired::scene::node::NodeRes};

use super::{
    bindings::wired::{
        math::types::{Quat, Transform, Vec3},
        scene::types::{Collider, HostNode, RigidBodyKind},
    },
    material::MaterialRes,
    mesh::MeshRes,
};

impl HostNode for Runtime {
    async fn id(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<String> {
        todo!()
    }

    async fn clone(&mut self, self_: Resource<NodeRes>) -> wasmtime::Result<Resource<NodeRes>> {
        todo!()
    }

    async fn name(&mut self, _self_: Resource<NodeRes>) -> wasmtime::Result<Option<String>> {
        Ok(None)
    }

    async fn set_name(
        &mut self,
        _self_: Resource<NodeRes>,
        _value: Option<String>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn translation(&mut self, _self_: Resource<NodeRes>) -> wasmtime::Result<Vec3> {
        todo!()
    }

    async fn set_translation(
        &mut self,
        _self_: Resource<NodeRes>,
        _value: Vec3,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn rotation(&mut self, _self_: Resource<NodeRes>) -> wasmtime::Result<Quat> {
        todo!()
    }

    async fn set_rotation(
        &mut self,
        _self_: Resource<NodeRes>,
        _value: Quat,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn scale(&mut self, _self_: Resource<NodeRes>) -> wasmtime::Result<Vec3> {
        todo!()
    }

    async fn set_scale(&mut self, _self_: Resource<NodeRes>, _value: Vec3) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn transform(&mut self, _self_: Resource<NodeRes>) -> wasmtime::Result<Transform> {
        todo!()
    }

    async fn set_transform(
        &mut self,
        _self_: Resource<NodeRes>,
        _value: Transform,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn global_transform(&mut self, _self_: Resource<NodeRes>) -> wasmtime::Result<Transform> {
        todo!()
    }

    async fn parent(
        &mut self,
        _self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Option<Resource<NodeRes>>> {
        Ok(None)
    }

    async fn children(
        &mut self,
        _self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Vec<Resource<NodeRes>>> {
        Ok(vec![])
    }

    async fn add_child(
        &mut self,
        _self_: Resource<NodeRes>,
        _child: Resource<NodeRes>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn remove_child(
        &mut self,
        _self_: Resource<NodeRes>,
        _child: Resource<NodeRes>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn mesh(
        &mut self,
        _self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Option<Resource<MeshRes>>> {
        Ok(None)
    }

    async fn set_mesh(
        &mut self,
        _self_: Resource<NodeRes>,
        _value: Option<Resource<MeshRes>>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn material(
        &mut self,
        _self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Option<Resource<MaterialRes>>> {
        Ok(None)
    }

    async fn set_material(
        &mut self,
        _self_: Resource<NodeRes>,
        _value: Option<Resource<MaterialRes>>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn collider(&mut self, _self_: Resource<NodeRes>) -> wasmtime::Result<Option<Collider>> {
        Ok(None)
    }

    async fn set_collider(
        &mut self,
        _self_: Resource<NodeRes>,
        _value: Option<Collider>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn rigid_body(
        &mut self,
        _self_: Resource<NodeRes>,
    ) -> wasmtime::Result<Option<RigidBodyKind>> {
        Ok(None)
    }

    async fn set_rigid_body(
        &mut self,
        _self_: Resource<NodeRes>,
        _value: Option<RigidBodyKind>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn sync(&mut self, _self_: Resource<NodeRes>) -> wasmtime::Result<bool> {
        Ok(false)
    }

    async fn set_sync(&mut self, _self_: Resource<NodeRes>, _value: bool) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<NodeRes>) -> wasmtime::Result<()> {
        self.native.table.delete(rep)?;
        Ok(())
    }
}
