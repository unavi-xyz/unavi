use std::{
    collections::HashMap,
    sync::Arc,
};

use bevy::prelude::*;
use bevy_vrm::BoneName;
use hsd::id::{
    DocId,
    PrimId,
};
use parking_lot::RwLock;
use unavi_agent::{
    Agent,
    AgentAvatar,
    AgentCamera,
    AgentDid,
    LocalAgent,
};
use unavi_avatar::bones::AvatarBones;
use xdid::core::did::Did;

use crate::runtime::shared::registry::transform::{
    AbsoluteNodeId,
    RegisterTransforms,
};

#[derive(Resource, Clone, Default)]
pub struct AgentProxyRegistry(Arc<RwLock<HashMap<AgentKey, AgentProxies>>>);

impl AgentProxyRegistry {
    #[must_use]
    pub fn contains(&self, key: &AgentKey) -> bool {
        self.0.read().contains_key(key)
    }

    pub fn insert(&self, key: AgentKey, proxies: AgentProxies) {
        self.0.write().insert(key, proxies);
    }

    pub fn remove(&self, key: &AgentKey) {
        self.0.write().remove(key);
    }

    /// The local agent's camera proxy, if it has registered one.
    #[must_use]
    pub fn camera(&self) -> Option<AbsoluteNodeId> {
        self.0.read().get(&AgentKey::Local)?.camera
    }

    #[must_use]
    pub fn bone(&self, key: &AgentKey, bone: BoneName) -> Option<AbsoluteNodeId> {
        self.0.read().get(key)?.bones.get(&bone).copied()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AgentKey {
    Local,
    Peer(Did),
}

#[derive(Default)]
pub struct AgentProxies {
    pub bones:  HashMap<BoneName, AbsoluteNodeId>,
    pub camera: Option<AbsoluteNodeId>,
}

#[derive(Component)]
pub struct RegisterAgent(AgentKey);

pub fn register_peers(
    trigger: On<Add, AgentDid>,
    agents: Query<&AgentDid, With<Agent>>,
    mut commands: Commands,
) {
    let Ok(did) = agents.get(trigger.entity) else {
        return;
    };
    commands
        .entity(trigger.entity)
        .insert(RegisterAgent(AgentKey::Peer(did.0.clone())));
}

pub fn register_local_agent(trigger: On<Add, LocalAgent>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(RegisterAgent(AgentKey::Local));
}

pub fn spawn_proxy_nodes(
    agents: Query<(&RegisterAgent, &AgentAvatar, Option<&AgentCamera>)>,
    avatars: Query<&AvatarBones>,
    registry: Res<AgentProxyRegistry>,
    mut commands: Commands,
) {
    for (key, avatar_ent, camera_ent) in agents {
        if registry.contains(&key.0) {
            continue;
        }

        let Ok(bones) = avatars.get(avatar_ent.0) else {
            continue;
        };

        let mut proxies = AgentProxies::default();

        if let Some(camera_ent) = camera_ent {
            let id = gen_proxy_id();
            proxies.camera = Some(id);
            let child = commands
                .spawn((
                    Name::new("camera"),
                    RegisterTransforms(id),
                    Visibility::default(),
                ))
                .id();
            commands.entity(camera_ent.0).add_child(child);
        }

        for (name, entity) in &bones.0 {
            let id = gen_proxy_id();
            proxies.bones.insert(*name, id);
            let child = commands
                .spawn((
                    Name::new(name.to_string()),
                    RegisterTransforms(id),
                    Visibility::default(),
                ))
                .id();
            commands.entity(*entity).add_child(child);
        }

        info!("Registering agent: {:?}", key.0);
        registry.insert(key.0.clone(), proxies);
    }
}

fn gen_proxy_id() -> AbsoluteNodeId {
    // Proxies are never referenced across peers; a fresh id under a blank
    // document is safe.
    AbsoluteNodeId {
        doc:  DocId([0; 32]),
        node: PrimId::new(),
    }
}

pub fn deregister_agents(
    trigger: On<Remove, RegisterAgent>,
    ids: Query<&RegisterAgent>,
    registry: Res<AgentProxyRegistry>,
) {
    let id = ids.get(trigger.entity).expect("id");
    registry.remove(&id.0);
    // Proxies will be cleaned up automatically on agent despawn.
}
