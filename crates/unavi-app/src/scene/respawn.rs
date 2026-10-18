use avian3d::prelude::{
    AngularVelocity,
    LinearVelocity,
};
use bevy::{
    ecs::system::SystemParam,
    prelude::*,
};
use bevy_hsd::attributes::spawn::SpawnPoint;
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
    config::AgentConfig,
};
use unavi_space::{
    grid::ActiveSpace,
    membership::Space,
    spawn::pick_spawn,
};

#[derive(Event, Default)]
pub struct Respawn {
    /// Space to spawn into; `None` uses the active space.
    pub space: Option<Entity>,
    /// World position to spawn at; `None` picks one in the space.
    pub at:    Option<Vec3>,
}

/// Where a space puts an arriving agent.
#[derive(SystemParam)]
pub struct SpawnPlaces<'w, 's> {
    spaces:       Query<'w, 's, &'static GlobalTransform, With<Space>>,
    spawn_points: Query<
        'w,
        's,
        (
            &'static SpawnPoint,
            &'static GlobalTransform,
            &'static ChildOf,
        ),
    >,
    parents:      Query<'w, 's, &'static ChildOf>,
}

impl SpawnPlaces<'_, '_> {
    /// A random point among the space's spawn points, else the space's own
    /// origin.
    #[must_use]
    pub fn pick(&self, space: Entity) -> Vec3 {
        pick_spawn(space, &self.spawn_points, &self.parents)
            .or_else(|| self.origin(space))
            .unwrap_or_default()
    }

    #[must_use]
    pub fn origin(&self, space: Entity) -> Option<Vec3> {
        self.spaces
            .get(space)
            .ok()
            .map(GlobalTransform::translation)
    }
}

pub fn respawn(
    trigger: On<Respawn>,
    local_agent: Query<(&LocalAgentEntities, &AgentConfig), With<LocalAgent>>,
    active: Res<ActiveSpace>,
    places: SpawnPlaces,
    mut body: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity)>,
) {
    let Ok((ents, config)) = local_agent.single() else {
        warn!("Can't respawn, local agent not found");
        return;
    };
    let Ok((mut tr, mut vel, mut ang_vel)) = body.get_mut(ents.body) else {
        return;
    };

    *vel = LinearVelocity::default();
    *ang_vel = AngularVelocity::default();

    // Spawn points name where the feet go, and the body's origin is its
    // capsule's centre, so it is set at its float height above the point.
    let event = trigger.event();
    let ground = event.at.unwrap_or_else(|| {
        event
            .space
            .or(active.0)
            .map_or_else(Vec3::default, |space| places.pick(space))
    });
    tr.translation = ground + Vec3::Y * config.float_height();

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
        commands.trigger(Respawn::default());
    }
}
