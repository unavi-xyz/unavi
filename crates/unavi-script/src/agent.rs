use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_hsd::{
    NodeId,
    cache::{NodeChanges, NodeInner, NodeState},
};
use bevy_vrm::BoneName;
use smol_str::SmolStr;
use unavi_agent::{Agent, LocalAgent, LocalAgentEntities};
use unavi_avatar::bones::AvatarBones;

use crate::{api::wired::scene::document::gen_id, load::LoadedScript, runtime::ScriptRuntime};

pub struct ProxyRegistry {
    pub bone_nodes: Arc<HashMap<BoneName, SmolStr>>,
    pub bone_proxy_ents: Arc<HashMap<BoneName, Entity>>,
}

#[derive(Component)]
pub struct BoneProxy;

#[derive(Component, Default)]
pub struct AgentProxies(pub Arc<Mutex<Vec<Arc<ProxyRegistry>>>>);

/// On a script entity while its agent proxy is not yet built.
#[derive(Component)]
pub struct NeedsAgentProxy(pub Entity); // points to LocalAgent entity

pub(crate) fn on_avatar_bones_added(
    trigger: On<Add, AvatarBones>,
    mut commands: Commands,
    child_of: Query<&ChildOf>,
    agent_markers: Query<(), With<Agent>>,
    agent_proxies: Query<&AgentProxies>,
    avatar_bones: Query<&AvatarBones>,
) {
    let avatar_ent = trigger.entity;
    let Ok(body) = child_of.get(avatar_ent) else {
        return;
    };
    let Ok(agent) = child_of.get(body.0) else {
        return;
    };
    let agent_ent = agent.0;
    if !agent_markers.contains(agent_ent) {
        info!("avatar grand parent not agent, ignoring");
        return;
    }

    let Ok(bones) = avatar_bones.get(avatar_ent) else {
        return;
    };

    if let Ok(ap) = agent_proxies.get(agent_ent) {
        info!("re-parenting agent proxies: {agent_ent}");
        let proxies = ap.0.lock().expect("proxies lock");
        for entry in proxies.iter() {
            for (&bone, &proxy_ent) in entry.bone_proxy_ents.iter() {
                if let Some(&bone_ent) = bones.get(&bone) {
                    commands.entity(proxy_ent).insert(ChildOf(bone_ent));
                }
            }
        }
    } else {
        commands.entity(agent_ent).insert(AgentProxies::default());
    }
}

pub(crate) fn init_agent_proxies(
    mut commands: Commands,
    pending: Query<(Entity, &NeedsAgentProxy, &ScriptRuntime), With<LoadedScript>>,
    local_agent: Query<(&LocalAgentEntities, Option<&AgentProxies>), With<LocalAgent>>,
    avatar_bones: Query<&AvatarBones>,
) {
    for (script_ent, needs_proxy, rt) in &pending {
        let Ok((entities, maybe_ap)) = local_agent.get(needs_proxy.0) else {
            continue;
        };
        let Ok(bones) = avatar_bones.get(entities.avatar) else {
            continue;
        };
        let Some(ap) = maybe_ap else {
            // AgentProxies not yet inserted by observer — retry next frame
            continue;
        };

        let Ok(mut ctx) = rt.ctx.try_lock() else {
            warn!("could not lock script ctx for proxy init — retry next frame");
            continue;
        };

        let self_node_id = ctx.store.data().rt.wired_scene.self_node_id.clone();
        let registry = Arc::clone(&ctx.store.data().rt.wired_scene.registry);

        let mut bone_nodes = HashMap::new();
        let mut bone_proxy_ents = HashMap::new();

        for (&bone, &bone_ent) in &bones.0 {
            let id = gen_id();
            bone_nodes.insert(bone, id.clone());
            let inner = Arc::new(NodeInner {
                changes: Mutex::new(NodeChanges::default()),
                entity: Mutex::new(None),
                id: id.clone(),
                is_virtual: true,
                state: Mutex::new(NodeState::default()),
                sync: false.into(),
                tree_id: Mutex::new(None),
            });
            registry
                .nodes
                .lock()
                .expect("nodes lock")
                .push(Arc::clone(&inner));
            registry
                .node_map
                .lock()
                .expect("node_map lock")
                .insert(id.clone(), Arc::clone(&inner));
            let proxy_ent = commands
                .spawn((
                    NodeId(id),
                    BoneProxy,
                    ChildOf(bone_ent),
                    Transform::IDENTITY,
                ))
                .id();
            *inner.entity.lock().expect("entity lock") = Some(proxy_ent);
            bone_proxy_ents.insert(bone, proxy_ent);
        }

        let self_inner = Arc::new(NodeInner {
            changes: Mutex::new(NodeChanges::default()),
            entity: Mutex::new(None),
            id: self_node_id.clone(),
            is_virtual: true,
            state: Mutex::new(NodeState::default()),
            sync: false.into(),
            tree_id: Mutex::new(None),
        });
        registry
            .nodes
            .lock()
            .expect("nodes lock")
            .push(Arc::clone(&self_inner));
        registry
            .node_map
            .lock()
            .expect("node_map lock")
            .insert(self_node_id.clone(), Arc::clone(&self_inner));
        let self_ent = commands.spawn(NodeId(self_node_id)).id();
        *self_inner.entity.lock().expect("entity lock") = Some(self_ent);

        let entry = Arc::new(ProxyRegistry {
            bone_nodes: Arc::new(bone_nodes),
            bone_proxy_ents: Arc::new(bone_proxy_ents),
        });
        ap.0.lock()
            .expect("agent proxies lock")
            .push(Arc::clone(&entry));

        ctx.store.data_mut().rt.wired_agent.local_agent = Some(Arc::clone(&entry));
        drop(ctx);

        commands.entity(script_ent).remove::<NeedsAgentProxy>();
    }
}

pub fn reset_bone_proxies(mut proxies: Query<&mut Transform, With<BoneProxy>>) {
    for mut t in &mut proxies {
        *t = Transform::IDENTITY;
    }
}
