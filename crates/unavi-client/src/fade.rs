use bevy::prelude::*;

const CLEAR_COLOR: Color = Color::BLACK;

pub struct FadePlugin;

impl Plugin for FadePlugin {
    fn build(&self, app: &mut App) {
        app.world_mut().spawn((
            FadeOverlay,
            FadeTimer {
                elapsed:  0.0,
                duration: 2.0,
                delay:    2.0,
            },
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(CLEAR_COLOR),
        ));

        app.insert_resource(ClearColor(CLEAR_COLOR))
            .add_systems(Update, update_fade);
    }
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
