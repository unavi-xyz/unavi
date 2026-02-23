use smol_str::SmolStr;

use crate::api::wired::scene::WiredSceneRt;

pub struct HostNode {
    id: SmolStr,
}

impl super::bindings::wired::scene::types::HostNode for WiredSceneRt {
    async fn name(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<Option<wasmtime::component::__internal::String>> {
        todo!()
    }
    async fn set_name(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: Option<wasmtime::component::__internal::String>,
    ) -> wasmtime::Result<()> {
        todo!()
    }

    async fn parent(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<Option<wasmtime::component::Resource<HostNode>>> {
        todo!()
    }
    async fn children(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<
        wasmtime::component::__internal::Vec<wasmtime::component::Resource<HostNode>>,
    > {
        todo!()
    }
    async fn add_child(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        child: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        todo!()
    }
    async fn remove_child(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        child: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<()> {
        todo!()
    }

    async fn rotation(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<super::bindings::wired::scene::types::Quat> {
        todo!()
    }
    async fn scale(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<super::bindings::wired::scene::types::Vec3> {
        todo!()
    }
    async fn transform(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<super::bindings::wired::scene::types::Transform> {
        todo!()
    }
    async fn translation(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<super::bindings::wired::scene::types::Vec3> {
        todo!()
    }
    async fn set_rotation(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: super::bindings::wired::scene::types::Quat,
    ) -> wasmtime::Result<()> {
        todo!()
    }
    async fn set_scale(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: super::bindings::wired::scene::types::Vec3,
    ) -> wasmtime::Result<()> {
        todo!()
    }
    async fn set_transform(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: super::bindings::wired::scene::types::Transform,
    ) -> wasmtime::Result<()> {
        todo!()
    }
    async fn set_translation(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: super::bindings::wired::scene::types::Vec3,
    ) -> wasmtime::Result<()> {
        todo!()
    }
    async fn global_transform(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<super::bindings::wired::scene::types::Transform> {
        todo!()
    }

    async fn material(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<
        Option<wasmtime::component::Resource<super::bindings::wired::scene::types::Material>>,
    > {
        todo!()
    }
    async fn set_material(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: Option<
            wasmtime::component::Resource<super::bindings::wired::scene::types::Material>,
        >,
    ) -> wasmtime::Result<()> {
        todo!()
    }

    async fn mesh(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<
        Option<wasmtime::component::Resource<super::bindings::wired::scene::types::Mesh>>,
    > {
        todo!()
    }
    async fn set_mesh(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: Option<wasmtime::component::Resource<super::bindings::wired::scene::types::Mesh>>,
    ) -> wasmtime::Result<()> {
        todo!()
    }

    async fn collider(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<Option<super::bindings::wired::scene::types::Collider>> {
        todo!()
    }
    async fn set_collider(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: Option<super::bindings::wired::scene::types::Collider>,
    ) -> wasmtime::Result<()> {
        todo!()
    }
    async fn rigid_body(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
    ) -> wasmtime::Result<Option<super::bindings::wired::scene::types::RigidBodyKind>> {
        todo!()
    }
    async fn set_rigid_body(
        &mut self,
        self_: wasmtime::component::Resource<HostNode>,
        value: Option<super::bindings::wired::scene::types::RigidBodyKind>,
    ) -> wasmtime::Result<()> {
        todo!()
    }

    async fn drop(&mut self, rep: wasmtime::component::Resource<HostNode>) -> wasmtime::Result<()> {
        todo!()
    }
}
