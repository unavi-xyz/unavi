use bevy::prelude::*;

/// Moving average of velocity, derived from changes in [`Transform`].
#[derive(Component)]
pub struct AverageVelocity {
    pub alpha:            f32,
    pub initialized:      bool,
    pub prev_translation: Vec3,
    /// The entity to track the velocity of.
    pub target:           Entity,
    pub velocity:         Vec3,
}

impl AverageVelocity {
    #[must_use]
    pub fn new(target: Entity) -> Self {
        Self {
            alpha: 0.4,
            initialized: false,
            prev_translation: Vec3::default(),
            target,
            velocity: Vec3::default(),
        }
    }
}

pub(crate) fn calc_average_velocity(
    mut velocities: Query<&mut AverageVelocity>,
    time: Res<Time>,
    transforms: Query<&Transform>,
) {
    let delta_t = time.delta_secs();
    // A paused or stalled clock divides by zero into NaN; skip the update
    // rather than feed that downstream.
    if delta_t <= f32::EPSILON {
        return;
    }

    for mut avg in &mut velocities {
        let Ok(transform) = transforms.get(avg.target) else {
            // The target despawned (agent left, remote avatar's body gone);
            // leave the last known velocity in place rather than panic.
            continue;
        };

        if !avg.initialized {
            avg.prev_translation.clone_from(&transform.translation);
            avg.initialized = true;
            continue;
        }

        let velocity = (transform.translation - avg.prev_translation) / delta_t;
        avg.prev_translation.clone_from(&transform.translation);

        avg.velocity.x = avg
            .alpha
            .mul_add(velocity.x, (1.0 - avg.alpha) * avg.velocity.x);
        avg.velocity.y = avg
            .alpha
            .mul_add(velocity.y, (1.0 - avg.alpha) * avg.velocity.y);
        avg.velocity.z = avg
            .alpha
            .mul_add(velocity.z, (1.0 - avg.alpha) * avg.velocity.z);
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    #[test]
    fn a_despawned_target_is_skipped_without_panicking() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(16));

        let target = app.world_mut().spawn(Transform::default()).id();
        app.world_mut().despawn(target);
        let tracker = app.world_mut().spawn(AverageVelocity::new(target)).id();

        app.world_mut()
            .run_system_once(calc_average_velocity)
            .expect("run once");

        let avg = app.world().get::<AverageVelocity>(tracker).expect("avg");
        assert!(!avg.initialized);
        assert_eq!(avg.velocity, Vec3::ZERO);
    }

    #[test]
    fn a_zero_delta_time_does_not_produce_nan() {
        let mut app = App::new();
        app.add_plugins(bevy::time::TimePlugin);
        // `Time` starts with a zero delta before the first `advance_by`.

        let target = app
            .world_mut()
            .spawn(Transform::from_xyz(1.0, 0.0, 0.0))
            .id();
        let tracker = app.world_mut().spawn(AverageVelocity::new(target)).id();

        app.world_mut()
            .run_system_once(calc_average_velocity)
            .expect("run once");

        let avg = app.world().get::<AverageVelocity>(tracker).expect("avg");
        assert!(avg.velocity.is_finite());
    }
}
