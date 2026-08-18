use wasmtime::component::Resource;

use crate::{
    error::ScriptError,
    runtime::{
        Runtime,
        native::wired::{
            error::bindings::wired::error::types::Error,
            scene::bindings::wired::{
                math::types::Transform as WitTransform,
                scene::types::HostDocument,
            },
        },
        shared::{
            self,
            wired::scene::{
                document::DocRes,
                prim::PrimRes,
            },
        },
    },
};

impl HostDocument for Runtime {
    async fn id(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<u8>> {
        shared::wired::scene::document::id(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn clone(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Resource<DocRes>> {
        shared::wired::scene::document::clone(&self.api, self_.rep())
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn roots(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<PrimRes>>> {
        shared::wired::scene::document::roots(&self.api, self_.rep())
            .await
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn prims(&mut self, self_: Resource<DocRes>) -> wasmtime::Result<Vec<Resource<PrimRes>>> {
        shared::wired::scene::document::prims(&self.api, self_.rep())
            .await
            .map(|v| v.into_iter().map(Resource::new_own).collect())
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn get_prim(
        &mut self,
        self_: Resource<DocRes>,
        id: String,
    ) -> wasmtime::Result<Option<Resource<PrimRes>>> {
        shared::wired::scene::document::get_prim(&self.api, self_.rep(), id)
            .await
            .map(|r| r.map(Resource::new_own))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn create_prim(
        &mut self,
        self_: Resource<DocRes>,
    ) -> wasmtime::Result<Result<Resource<PrimRes>, Error>> {
        Ok(
            shared::wired::scene::document::create_prim(&self.api, self_.rep())
                .await
                .map(Resource::new_own)
                .map_err(|e| ScriptError::from(e).into()),
        )
    }

    async fn remove_prim(
        &mut self,
        _self_: Resource<DocRes>,
        value: Resource<PrimRes>,
    ) -> wasmtime::Result<Result<(), Error>> {
        Ok(
            shared::wired::scene::document::remove_prim(&self.api, value.rep())
                .await
                .map_err(|e| ScriptError::from(e).into()),
        )
    }

    async fn offset_to(
        &mut self,
        self_: Resource<DocRes>,
        other: Resource<DocRes>,
    ) -> wasmtime::Result<Result<Option<WitTransform>, Error>> {
        Ok(
            shared::wired::scene::document::offset_to(&self.api, self_.rep(), other.rep())
                .await
                .map(|opt| {
                    opt.map(|x| WitTransform {
                        translation: bevy::math::Vec3::from_array(x.translation).into(),
                        rotation:    bevy::math::Quat::from_array(x.rotation).into(),
                        scale:       bevy::math::Vec3::from_array(x.scale).into(),
                    })
                })
                .map_err(|err| ScriptError::from(err).into()),
        )
    }

    async fn set_anchor(
        &mut self,
        self_: Resource<DocRes>,
        target: Option<Resource<PrimRes>>,
    ) -> wasmtime::Result<Result<(), Error>> {
        let target = target.map(|t| t.rep());
        Ok(
            shared::wired::scene::document::set_anchor(&self.api, self_.rep(), target)
                .await
                .map_err(|e| ScriptError::from(e).into()),
        )
    }

    async fn set_offset(
        &mut self,
        self_: Resource<DocRes>,
        value: WitTransform,
    ) -> wasmtime::Result<Result<(), Error>> {
        let value = shared::wired::scene::document::XformValue {
            translation: [
                value.translation.x,
                value.translation.y,
                value.translation.z,
            ],
            rotation:    [
                value.rotation.x,
                value.rotation.y,
                value.rotation.z,
                value.rotation.w,
            ],
            scale:       [value.scale.x, value.scale.y, value.scale.z],
        };
        Ok(
            shared::wired::scene::document::set_offset(&self.api, self_.rep(), value)
                .await
                .map_err(|e| ScriptError::from(e).into()),
        )
    }

    async fn drop(&mut self, rep: Resource<DocRes>) -> wasmtime::Result<()> {
        shared::wired::scene::document::on_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}
