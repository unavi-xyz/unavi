use bevy::prelude::*;

/// Calculates the exponential moving average (EMA) of the velocity,
/// via changes in [Transform].
#[derive(Component)]
pub struct AverageVelocity {
    initialized: bool,
    prev_translation: Vec3,
    pub alpha: f32,
    pub velocity: Vec3,
}

impl Default for AverageVelocity {
    fn default() -> Self {
        Self {
            alpha: 0.1,
            initialized: false,
            prev_translation: Vec3::default(),
            velocity: Vec3::default(),
        }
    }
}

pub fn calc_average_velocity(
    mut velocities: Query<(&mut AverageVelocity, &Transform)>,
    time: Res<Time>,
) {
    let delta_t = time.delta_seconds();

    for (mut avg, transform) in velocities.iter_mut() {
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
