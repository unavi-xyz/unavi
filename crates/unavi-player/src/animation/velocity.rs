use bevy::prelude::*;

/// Calculates the exponential moving average (EMA) of the velocity,
/// via changes in [Transform].
#[derive(Component)]
pub struct AverageVelocity {
    pub alpha: f32,
    pub initialized: bool,
    pub prev_translation: Vec3,
    /// The target entity to track the velocity of.
    /// If set to None, the current entity will be used.
    pub target: Option<Entity>,
    pub velocity: Vec3,
}

impl Default for AverageVelocity {
    fn default() -> Self {
        Self {
            alpha: 0.1,
            initialized: false,
            prev_translation: Vec3::default(),
            target: None,
            velocity: Vec3::default(),
        }
    }
}

pub fn calc_average_velocity(
    mut velocities: Query<(Entity, &mut AverageVelocity)>,
    time: Res<Time>,
    transforms: Query<&Transform>,
) {
    let delta_t = time.delta_secs();

    for (entity, mut avg) in &mut velocities {
        let target = avg.target.unwrap_or(entity);

        let transform = transforms
            .get(target)
            .expect("Velocity target has no transform");

        if !avg.initialized {
            avg.prev_translation.clone_from(&transform.translation);
            avg.initialized = true;
            continue;
        }

        let velocity = (transform.translation - avg.prev_translation) / delta_t;
        avg.prev_translation.clone_from(&transform.translation);

        avg.velocity.x = avg.alpha * velocity.x + (1.0 - avg.alpha) * avg.velocity.x;
        avg.velocity.y = avg.alpha * velocity.y + (1.0 - avg.alpha) * avg.velocity.y;
        avg.velocity.z = avg.alpha * velocity.z + (1.0 - avg.alpha) * avg.velocity.z;
    }
}
