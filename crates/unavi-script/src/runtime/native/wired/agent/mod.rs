use bevy_vrm::BoneName;
use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        wired::{agent::AgentRes, scene::prim::PrimRes},
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::{agent::AgentRes, scene::prim::PrimRes};

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-agent",
        with: {
            "wired:agent/types.agent": AgentRes,
            "wired:scene/types.prim": PrimRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::agent::types::{BoneName as WitBoneName, HostAgent};

impl From<WitBoneName> for BoneName {
    fn from(b: WitBoneName) -> Self {
        match b {
            WitBoneName::Hips => Self::Hips,
            WitBoneName::Spine => Self::Spine,
            WitBoneName::Chest => Self::Chest,
            WitBoneName::UpperChest => Self::UpperChest,
            WitBoneName::Neck => Self::Neck,
            WitBoneName::Head => Self::Head,
            WitBoneName::LeftEye => Self::LeftEye,
            WitBoneName::RightEye => Self::RightEye,
            WitBoneName::Jaw => Self::Jaw,
            WitBoneName::LeftShoulder => Self::LeftShoulder,
            WitBoneName::LeftUpperArm => Self::LeftUpperArm,
            WitBoneName::LeftLowerArm => Self::LeftLowerArm,
            WitBoneName::LeftHand => Self::LeftHand,
            WitBoneName::RightShoulder => Self::RightShoulder,
            WitBoneName::RightUpperArm => Self::RightUpperArm,
            WitBoneName::RightLowerArm => Self::RightLowerArm,
            WitBoneName::RightHand => Self::RightHand,
            WitBoneName::LeftUpperLeg => Self::LeftUpperLeg,
            WitBoneName::LeftLowerLeg => Self::LeftLowerLeg,
            WitBoneName::LeftFoot => Self::LeftFoot,
            WitBoneName::LeftToes => Self::LeftToes,
            WitBoneName::RightUpperLeg => Self::RightUpperLeg,
            WitBoneName::RightLowerLeg => Self::RightLowerLeg,
            WitBoneName::RightFoot => Self::RightFoot,
            WitBoneName::RightToes => Self::RightToes,
            WitBoneName::LeftThumbProximal => Self::LeftThumbProximal,
            WitBoneName::LeftThumbIntermediate => Self::LeftThumbIntermediate,
            WitBoneName::LeftThumbDistal => Self::LeftThumbDistal,
            WitBoneName::LeftIndexProximal => Self::LeftIndexProximal,
            WitBoneName::LeftIndexIntermediate => Self::LeftIndexIntermediate,
            WitBoneName::LeftIndexDistal => Self::LeftIndexDistal,
            WitBoneName::LeftMiddleProximal => Self::LeftMiddleProximal,
            WitBoneName::LeftMiddleIntermediate => Self::LeftMiddleIntermediate,
            WitBoneName::LeftMiddleDistal => Self::LeftMiddleDistal,
            WitBoneName::LeftRingProximal => Self::LeftRingProximal,
            WitBoneName::LeftRingIntermediate => Self::LeftRingIntermediate,
            WitBoneName::LeftRingDistal => Self::LeftRingDistal,
            WitBoneName::LeftLittleProximal => Self::LeftLittleProximal,
            WitBoneName::LeftLittleIntermediate => Self::LeftLittleIntermediate,
            WitBoneName::LeftLittleDistal => Self::LeftLittleDistal,
            WitBoneName::RightThumbProximal => Self::RightThumbProximal,
            WitBoneName::RightThumbIntermediate => Self::RightThumbIntermediate,
            WitBoneName::RightThumbDistal => Self::RightThumbDistal,
            WitBoneName::RightIndexProximal => Self::RightIndexProximal,
            WitBoneName::RightIndexIntermediate => Self::RightIndexIntermediate,
            WitBoneName::RightIndexDistal => Self::RightIndexDistal,
            WitBoneName::RightMiddleProximal => Self::RightMiddleProximal,
            WitBoneName::RightMiddleIntermediate => Self::RightMiddleIntermediate,
            WitBoneName::RightMiddleDistal => Self::RightMiddleDistal,
            WitBoneName::RightRingProximal => Self::RightRingProximal,
            WitBoneName::RightRingIntermediate => Self::RightRingIntermediate,
            WitBoneName::RightRingDistal => Self::RightRingDistal,
            WitBoneName::RightLittleProximal => Self::RightLittleProximal,
            WitBoneName::RightLittleIntermediate => Self::RightLittleIntermediate,
            WitBoneName::RightLittleDistal => Self::RightLittleDistal,
        }
    }
}

impl bindings::wired::agent::types::Host for Runtime {}

impl HostAgent for Runtime {
    async fn bone(
        &mut self,
        self_: Resource<AgentRes>,
        name: WitBoneName,
    ) -> wasmtime::Result<Option<Resource<PrimRes>>> {
        shared::wired::agent::bone(&self.api, self_.rep(), name.into())
            .await
            .map(|opt| opt.map(Resource::new_own))
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<AgentRes>) -> wasmtime::Result<()> {
        shared::wired::agent::on_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl bindings::wired::agent::api::Host for Runtime {
    async fn local_agent(&mut self) -> wasmtime::Result<Resource<AgentRes>> {
        shared::wired::agent::local_agent(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn local_camera(&mut self) -> wasmtime::Result<Resource<PrimRes>> {
        shared::wired::agent::local_camera(&self.api)
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }
}
