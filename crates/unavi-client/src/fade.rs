use bevy::prelude::*;

pub struct FadePlugin;

impl Plugin for FadePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_fade_overlay)
            .add_systems(Update, update_fade);
    }
}

#[derive(Component)]
struct FadeOverlay;

#[derive(Component)]
struct FadeTimer {
    elapsed: f32,
    duration: f32,
    delay: f32,
}

fn spawn_fade_overlay(mut commands: Commands) {
    // Spawn full-screen black overlay.
    commands.spawn((
        FadeOverlay,
        FadeTimer {
            elapsed: 0.0,
            duration: 2.0,
            delay: 1.0,
        },
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK),
    ));
}

fn update_fade(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FadeTimer, &mut BackgroundColor), With<FadeOverlay>>,
    time: Res<Time>,
) {
    for (entity, mut timer, mut bg) in &mut query {
        timer.elapsed += time.delta_secs();

        // Don't start fading until delay has passed.
        if timer.elapsed < timer.delay {
            continue;
        }

        // Calculate progress relative to the delay.
        let fade_elapsed = timer.elapsed - timer.delay;
        let progress = (fade_elapsed / timer.duration).min(1.0);
        let alpha = 1.0 - progress;

        bg.0.set_alpha(alpha);

        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}
