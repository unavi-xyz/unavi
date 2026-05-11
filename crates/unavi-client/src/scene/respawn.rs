use avian3d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;
use unavi_agent::{LocalAgent, LocalAgentEntities};

#[derive(Event)]
pub struct Respawn;

pub fn respawn(
    _: On<Respawn>,
    local_agent: Query<&LocalAgentEntities, With<LocalAgent>>,
    mut body: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
) {
    let Ok(ents) = local_agent.single() else {
        warn!("Can't respawn, local agent not found");
        return;
    };
    let Ok((mut tr, mut vel, mut ang_vel)) = body.get_mut(ents.body) else {
        return;
    };

    info!("Respawn");

    *vel = LinearVelocity::default();
    *ang_vel = AngularVelocity::default();

    // For now, just use origin as respawn point.
    tr.translation = Vec3::default();
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
