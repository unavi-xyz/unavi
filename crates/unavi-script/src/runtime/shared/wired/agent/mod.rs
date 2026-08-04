use std::sync::Arc;

use bevy_vrm::BoneName;

use crate::runtime::shared::{
    Api,
    registry::agent::{
        AGENT_REGISTRY,
        AgentKey,
    },
    slot_map::SlotMap,
    wired::scene::prim::PrimRes,
};

pub struct AgentRes {
    key: AgentKey,
}

#[derive(Default)]
pub struct WiredAgentApi {
    pub agents: SlotMap<AgentRes>,
}

pub async fn local_agent(api: &Api) -> anyhow::Result<u32> {
    Ok(api.wired_agent.lock().await.agents.insert(
        AgentRes {
            key: AgentKey::Local,
        },
        &api.quota,
    )?)
}

pub async fn local_camera(api: &Api) -> anyhow::Result<u32> {
    let (doc_id, node_id) = {
        let guard = AGENT_REGISTRY.read();
        let entry = guard
            .get(&AgentKey::Local)
            .ok_or_else(|| anyhow::anyhow!("agent entry not found"))?;
        let id = entry
            .camera
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("camera proxy not found"))?;
        let out = (id.doc, id.node);
        drop(guard);
        out
    };
    let rep = api.wired_scene.lock().await.prims.insert(
        PrimRes {
            state: Arc::default(),
            doc_id,
            id: node_id,
            is_proxy: true,
        },
        &api.quota,
    )?;
    Ok(rep)
}

pub async fn bone(api: &Api, rep: u32, name: BoneName) -> anyhow::Result<Option<u32>> {
    let key = {
        let wired_agent = api.wired_agent.lock().await;
        wired_agent
            .agents
            .get(rep)
            .ok_or_else(|| anyhow::anyhow!("resource not found"))?
            .key
            .clone()
    };
    let absolute = {
        let guard = AGENT_REGISTRY.read();
        let entry = guard
            .get(&key)
            .ok_or_else(|| anyhow::anyhow!("agent entry not found"))?;
        let out = entry.bones.get(&name).map(|id| (id.doc, id.node));
        drop(guard);
        out
    };
    if let Some((doc_id, node_id)) = absolute {
        let rep = api.wired_scene.lock().await.prims.insert(
            PrimRes {
                state: Arc::default(),
                doc_id,
                id: node_id,
                is_proxy: true,
            },
            &api.quota,
        )?;
        Ok(Some(rep))
    } else {
        Ok(None)
    }
}

pub async fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_agent.lock().await.agents.remove(rep);
    Ok(())
}
