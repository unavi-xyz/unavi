//! A black overlay that fades out on startup, masking the first frames while
//! the scene underneath is still assembling.

use bevy::prelude::*;

const CLEAR_COLOR: Color = Color::BLACK;

/// How long the overlay holds fully opaque before it starts fading, giving
/// the scene time to spawn in behind it.
const FADE_DELAY: f32 = 2.0;
/// How long the fade-out itself takes once it starts.
const FADE_DURATION: f32 = 1.0;

pub struct FadePlugin;

impl Plugin for FadePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(CLEAR_COLOR))
            .add_systems(Startup, spawn_overlay)
            .add_systems(Update, update_fade);
    }
}

fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        FadeOverlay,
        FadeTimer {
            elapsed:  0.0,
            duration: FADE_DURATION,
            delay:    FADE_DELAY,
        },
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(CLEAR_COLOR),
    ));
}

#[derive(Component)]
struct FadeOverlay;

#[derive(Component)]
struct FadeTimer {
    elapsed:  f32,
    duration: f32,
    delay:    f32,
}

fn update_fade(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FadeTimer, &mut BackgroundColor), With<FadeOverlay>>,
    time: Res<Time>,
) {
    for (entity, mut timer, mut bg) in &mut query {
        timer.elapsed += time.delta_secs();

        let fade_elapsed = timer.elapsed - timer.delay;
        if fade_elapsed < 0.0 {
            continue;
        }

        let progress = (fade_elapsed / timer.duration).min(1.0);
        let alpha = 1.0 - progress;

        bg.0.set_alpha(alpha);

        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}
