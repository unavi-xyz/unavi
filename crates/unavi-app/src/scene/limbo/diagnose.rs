//! Temporary trace of the agent body while in limbo, to find what moves it
//! off the held point. Remove once the floor glitch is understood.

use avian3d::prelude::*;
use bevy::prelude::*;
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
    config::AgentConfig,
};

use crate::scene::limbo::Limbo;

const TRACE_INTERVAL_SECS: f32 = 0.25;

#[derive(Default)]
pub struct TraceClock(f32);

pub fn trace_limbo_body(
    time: Res<Time>,
    mut clock: Local<TraceClock>,
    agents: Query<(&LocalAgentEntities, &AgentConfig), With<LocalAgent>>,
    floor: Query<&Transform, With<Limbo>>,
    bodies: Query<(
        &Transform,
        &Position,
        &LinearVelocity,
        Option<&CollidingEntities>,
    )>,
    names: Query<NameOrEntity>,
    mut commands: Commands,
) {
    let Ok((ents, config)) = agents.single() else {
        return;
    };
    let Ok((tr, pos, vel, colliding)) = bodies.get(ents.body) else {
        return;
    };
    let Some(colliding) = colliding else {
        commands.entity(ents.body).insert(CollidingEntities::default());
        return;
    };

    clock.0 += time.delta_secs();
    if clock.0 < TRACE_INTERVAL_SECS {
        return;
    }
    clock.0 = 0.0;

    let floor_y = floor.single().map(|floor| floor.translation.y).ok();
    let touching = colliding
        .iter()
        .map(|&entity| {
            names
                .get(entity)
                .map_or_else(|_| format!("{entity}"), |name| name.to_string())
        })
        .collect::<Vec<_>>();
    info!(
        target: "limbo_trace",
        transform_y = tr.translation.y,
        position_y = pos.0.y,
        velocity = ?vel.0,
        floor_y = ?floor_y,
        float_height = config.float_height(),
        ?touching,
        "limbo body"
    );
}
