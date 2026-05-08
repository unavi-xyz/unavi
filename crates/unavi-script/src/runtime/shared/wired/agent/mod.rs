use std::sync::Arc;

use bevy_vrm::BoneName;

use crate::runtime::shared::{
    Api,
    registry::agent::{AGENT_REGISTRY, AgentKey},
    slot_map::SlotMap,
    wired::scene::node::NodeRes,
};

pub mod bridge;

pub struct AgentRes {
    key: AgentKey,
}

#[derive(Default)]
pub struct WiredAgentApi {
    pub agents: SlotMap<AgentRes>,
}

pub fn local_agent(api: &Api) -> anyhow::Result<u32> {
    Ok(api.wired_agent.try_lock()?.agents.insert(AgentRes {
        key: AgentKey::Local,
    }))
}

pub fn local_camera(api: &Api) -> anyhow::Result<u32> {
    let entry = AGENT_REGISTRY
        .get_sync(&AgentKey::Local)
        .ok_or_else(|| anyhow::anyhow!("agent entry not found"))?;
    let id = entry
        .camera
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("camera proxy not found"))?;
    let rep = api.wired_scene.try_lock()?.nodes.insert(NodeRes {
        doc: Arc::default(),
        doc_id: id.doc,
        id: id.node,
        is_proxy: true,
    });
    Ok(rep)
}

pub fn bone(api: &Api, rep: u32, name: BoneName) -> anyhow::Result<Option<u32>> {
    let wired_agent = api.wired_agent.try_lock()?;
    let res = wired_agent
        .agents
        .get(rep)
        .ok_or_else(|| anyhow::anyhow!("resource not found"))?;
    let entry = AGENT_REGISTRY
        .get_sync(&res.key)
        .ok_or_else(|| anyhow::anyhow!("agent entry not found"))?;
    drop(wired_agent);

    if let Some(id) = entry.bones.get(&name) {
        let rep = api.wired_scene.try_lock()?.nodes.insert(NodeRes {
            doc: Arc::default(),
            doc_id: id.doc,
            id: id.node,
            is_proxy: true,
        });
        Ok(Some(rep))
    } else {
        Ok(None)
    }
}

pub fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_agent.try_lock()?.agents.remove(rep);
    Ok(())
}
