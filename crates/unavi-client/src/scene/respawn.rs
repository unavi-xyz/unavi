use avian3d::prelude::{
    AngularVelocity,
    LinearVelocity,
};
use bevy::prelude::*;
use bevy_hsd::attributes::spawn::SpawnPoint;
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
};
use unavi_space::{
    Space,
    anchor::ActiveSpace,
    spawn::pick_spawn,
};

#[derive(Event)]
pub struct Respawn;

pub fn respawn(
    _: On<Respawn>,
    local_agent: Query<&LocalAgentEntities, With<LocalAgent>>,
    active: Res<ActiveSpace>,
    spaces: Query<&GlobalTransform, With<Space>>,
    spawn_points: Query<(&SpawnPoint, &GlobalTransform, &ChildOf)>,
    parents: Query<&ChildOf>,
    mut body: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
) {
    let Ok(ents) = local_agent.single() else {
        warn!("Can't respawn, local agent not found");
        return;
    };
    let Ok((mut tr, mut vel, mut ang_vel)) = body.get_mut(ents.body) else {
        return;
    };

    *vel = LinearVelocity::default();
    *ang_vel = AngularVelocity::default();

    tr.translation = active.0.map_or_else(Vec3::default, |space| {
        pick_spawn(space, &spawn_points, &parents, &spaces).unwrap_or_default()
    });

    info!("Respawning at {}", tr.translation);
}

const VOID_LEVEL: f32 = -512.0;

pub fn teleport_from_void(
    local_agent: Query<&LocalAgentEntities, With<LocalAgent>>,
    transforms: Query<&Transform>,
    mut commands: Commands,
) {
    let Ok(ents) = local_agent.single() else {
        return;
    };
    let Ok(tr) = transforms.get(ents.body) else {
        return;
    };
    if tr.translation.y < VOID_LEVEL {
        info!(
            VOID_LEVEL,
            y = tr.translation.y,
            "Local agent fell into void, respawning"
        );
        commands.trigger(Respawn);
    }
}
