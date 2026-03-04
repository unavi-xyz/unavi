use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy_vrm::BoneName;
use unavi_avatar::{AvatarBones, AvatarBonesPopulated};
use unavi_locomotion::{AgentEntities, LocalAgent};
use unavi_script::agent::LocalAgentDocs;

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init_local_agent_docs);
    }
}

/// Creates the `LocalAgentDocs` resource when the local avatar's bones populate.
/// Does not create any `LoroDoc` — scripts create per-script docs in `load_scripts`.
pub fn init_local_agent_docs(
    mut commands: Commands,
    local_agents: Query<&AgentEntities, With<LocalAgent>>,
    avatars: Query<&AvatarBones, Added<AvatarBonesPopulated>>,
    existing: Option<Res<LocalAgentDocs>>,
) {
    if existing.is_some() {
        return;
    }
    let Ok(agent_ents) = local_agents.single() else {
        return;
    };
    let Ok(bones) = avatars.get(agent_ents.avatar) else {
        return;
    };

    let bone_entities: HashMap<BoneName, Entity> = bones.iter().map(|(&b, &e)| (b, e)).collect();
    commands.insert_resource(LocalAgentDocs {
        bone_entities: Arc::new(bone_entities),
        docs: Arc::new(Mutex::new(vec![])),
    });
}
