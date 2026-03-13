use std::sync::Arc;

use bevy_vrm::BoneName;
use wasmtime::component::{Resource, ResourceTable};

use crate::{
    agent::AgentDocEntry,
    api::wired::scene::{document::HostDocument, node::HostNode},
};

#[derive(Default)]
pub struct WiredAgentRt {
    pub local_agent: Option<Arc<AgentDocEntry>>,
    pub table: ResourceTable,
}

pub struct HostAgent(pub Arc<AgentDocEntry>);

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-agent",
        with: {
            "wired:scene/types.node":
                crate::api::wired::scene::node::HostNode,
            "wired:scene/types.document":
                crate::api::wired::scene::document::HostDocument,
            "wired:agent/types.agent": super::HostAgent,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

const fn wit_bone_to_vrm(b: bindings::wired::agent::types::BoneName) -> BoneName {
    use bindings::wired::agent::types::BoneName as WB;
    match b {
        WB::Hips => BoneName::Hips,
        WB::Spine => BoneName::Spine,
        WB::Chest => BoneName::Chest,
        WB::UpperChest => BoneName::UpperChest,
        WB::Neck => BoneName::Neck,
        WB::Head => BoneName::Head,
        WB::LeftEye => BoneName::LeftEye,
        WB::RightEye => BoneName::RightEye,
        WB::Jaw => BoneName::Jaw,
        WB::LeftShoulder => BoneName::LeftShoulder,
        WB::LeftUpperArm => BoneName::LeftUpperArm,
        WB::LeftLowerArm => BoneName::LeftLowerArm,
        WB::LeftHand => BoneName::LeftHand,
        WB::RightShoulder => BoneName::RightShoulder,
        WB::RightUpperArm => BoneName::RightUpperArm,
        WB::RightLowerArm => BoneName::RightLowerArm,
        WB::RightHand => BoneName::RightHand,
        WB::LeftUpperLeg => BoneName::LeftUpperLeg,
        WB::LeftLowerLeg => BoneName::LeftLowerLeg,
        WB::LeftFoot => BoneName::LeftFoot,
        WB::LeftToes => BoneName::LeftToes,
        WB::RightUpperLeg => BoneName::RightUpperLeg,
        WB::RightLowerLeg => BoneName::RightLowerLeg,
        WB::RightFoot => BoneName::RightFoot,
        WB::RightToes => BoneName::RightToes,
        WB::LeftThumbProximal => BoneName::LeftThumbProximal,
        WB::LeftThumbIntermediate => BoneName::LeftThumbIntermediate,
        WB::LeftThumbDistal => BoneName::LeftThumbDistal,
        WB::LeftIndexProximal => BoneName::LeftIndexProximal,
        WB::LeftIndexIntermediate => BoneName::LeftIndexIntermediate,
        WB::LeftIndexDistal => BoneName::LeftIndexDistal,
        WB::LeftMiddleProximal => BoneName::LeftMiddleProximal,
        WB::LeftMiddleIntermediate => BoneName::LeftMiddleIntermediate,
        WB::LeftMiddleDistal => BoneName::LeftMiddleDistal,
        WB::LeftRingProximal => BoneName::LeftRingProximal,
        WB::LeftRingIntermediate => BoneName::LeftRingIntermediate,
        WB::LeftRingDistal => BoneName::LeftRingDistal,
        WB::LeftLittleProximal => BoneName::LeftLittleProximal,
        WB::LeftLittleIntermediate => BoneName::LeftLittleIntermediate,
        WB::LeftLittleDistal => BoneName::LeftLittleDistal,
        WB::RightThumbProximal => BoneName::RightThumbProximal,
        WB::RightThumbIntermediate => BoneName::RightThumbIntermediate,
        WB::RightThumbDistal => BoneName::RightThumbDistal,
        WB::RightIndexProximal => BoneName::RightIndexProximal,
        WB::RightIndexIntermediate => BoneName::RightIndexIntermediate,
        WB::RightIndexDistal => BoneName::RightIndexDistal,
        WB::RightMiddleProximal => BoneName::RightMiddleProximal,
        WB::RightMiddleIntermediate => BoneName::RightMiddleIntermediate,
        WB::RightMiddleDistal => BoneName::RightMiddleDistal,
        WB::RightRingProximal => BoneName::RightRingProximal,
        WB::RightRingIntermediate => BoneName::RightRingIntermediate,
        WB::RightRingDistal => BoneName::RightRingDistal,
        WB::RightLittleProximal => BoneName::RightLittleProximal,
        WB::RightLittleIntermediate => BoneName::RightLittleIntermediate,
        WB::RightLittleDistal => BoneName::RightLittleDistal,
    }
}

use crate::load::state::RuntimeData;

impl bindings::wired::agent::context::Host for RuntimeData {
    async fn local_agent(&mut self) -> wasmtime::Result<Resource<HostAgent>> {
        let Some(entry) = self.wired_agent.local_agent.clone() else {
            return Err(anyhow::anyhow!("no local agent available"));
        };
        Ok(self.wired_agent.table.push(HostAgent(entry))?)
    }
}

impl bindings::wired::agent::types::Host for RuntimeData {}

impl bindings::wired::agent::types::HostAgent for RuntimeData {
    async fn document(
        &mut self,
        _self_: Resource<HostAgent>,
    ) -> wasmtime::Result<Resource<HostDocument>> {
        Ok(self.wired_scene.table.push(HostDocument {
            id: self.wired_scene.doc_id,
        })?)
    }

    async fn bone(
        &mut self,
        self_: Resource<HostAgent>,
        name: bindings::wired::agent::types::BoneName,
    ) -> wasmtime::Result<Option<Resource<HostNode>>> {
        let vrm_bone = wit_bone_to_vrm(name);
        let inner = {
            let agent = self.wired_agent.table.get(&self_)?;
            let Some(node_id) = agent.0.bone_nodes.get(&vrm_bone).cloned() else {
                return Ok(None);
            };
            let node_map = agent.0.registry.node_map.lock().expect("node_map lock");
            node_map.get(&node_id).cloned()
        };
        let Some(inner) = inner else {
            return Ok(None);
        };
        Ok(Some(self.wired_scene.table.push(HostNode { inner })?))
    }

    async fn drop(&mut self, rep: Resource<HostAgent>) -> wasmtime::Result<()> {
        self.wired_agent.table.delete(rep)?;
        Ok(())
    }
}
