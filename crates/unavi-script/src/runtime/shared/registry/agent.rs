use std::sync::LazyLock;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_vrm::BoneName;
use blake3::Hash;
use loro::TreeID;
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

pub static AGENT_REGISTRY: LazyLock<parking_lot::RwLock<HashMap<AgentKey, AgentProxies>>> =
    LazyLock::new(|| parking_lot::RwLock::new(HashMap::new()));

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
    mut commands: Commands,
) {
    for (key, avatar_ent, camera_ent) in agents {
        if AGENT_REGISTRY.read().contains_key(&key.0) {
            continue;
        }

        let Ok(bones) = avatars.get(avatar_ent.0) else {
            continue;
        };

        let mut proxies = AgentProxies::default();

        if let Some(camera_ent) = camera_ent {
            let id = gen_proxy_id();
            proxies.camera = Some(id.clone());
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
            proxies.bones.insert(*name, id.clone());
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
        AGENT_REGISTRY.write().insert(key.0.clone(), proxies);
    }
}

fn gen_proxy_id() -> AbsoluteNodeId {
    // The actual ID for proxy nodes isn't important.
    // Generate a random node ID, with a blank document hash.
    let peer = rand::random();
    let counter = rand::random();
    AbsoluteNodeId {
        doc:  Hash::from_bytes([0; 32]),
        node: TreeID::new(peer, counter),
    }
}

pub fn deregister_agents(trigger: On<Remove, RegisterAgent>, ids: Query<&RegisterAgent>) {
    let id = ids.get(trigger.entity).expect("id");
    AGENT_REGISTRY.write().remove(&id.0);
    // Proxies will be cleaned up automatically on agent despawn.
}
