use avian3d::prelude::PhysicsGizmos;
use bevy::{
    dev_tools::fps_overlay::FpsOverlayConfig,
    feathers::controls::toggle_switch,
    prelude::*,
    ui_widgets::{
        ValueChange,
        checkbox_self_update,
    },
};
use unavi_devtools::tabs::DevPanel;

#[derive(Component, Clone, Copy)]
pub(super) enum Toggle {
    Fps,
    Inspector,
    Physics,
    Events,
}

#[derive(Resource, Default)]
pub struct DevToggles([bool; 4]);

impl DevToggles {
    pub(super) const fn get(&self, toggle: Toggle) -> bool {
        self.0[toggle as usize]
    }
}

/// Builds a run condition that fires while the given toggle is on.
pub(super) fn toggled(toggle: Toggle) -> impl Fn(Res<DevToggles>) -> bool + Clone {
    move |toggles: Res<DevToggles>| toggles.get(toggle)
}

pub(super) fn spawn(mut commands: Commands) {
    commands
        .spawn((
            DevPanel {
                title: "Bevy".into(),
            },
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|panel| {
            for (label, toggle) in [
                ("FPS overlay", Toggle::Fps),
                ("World inspector", Toggle::Inspector),
                ("Physics gizmos", Toggle::Physics),
                ("Event gizmos", Toggle::Events),
            ] {
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        row.spawn(toggle_switch(toggle))
                            .observe(checkbox_self_update);
                    });
            }
        });
}

pub(super) fn on_toggle(
    change: On<ValueChange<bool>>,
    toggles: Query<&Toggle>,
    mut state: ResMut<DevToggles>,
) {
    if let Ok(toggle) = toggles.get(change.source) {
        state.0[*toggle as usize] = change.value;
    }
}

pub(super) fn apply_toggles(
    toggles: Res<DevToggles>,
    mut fps: ResMut<FpsOverlayConfig>,
    mut gizmos: ResMut<GizmoConfigStore>,
) {
    let want_fps = toggles.get(Toggle::Fps);
    if fps.enabled != want_fps {
        fps.enabled = want_fps;
    }
    if fps.frame_time_graph_config.enabled != want_fps {
        fps.frame_time_graph_config.enabled = want_fps;
    }
    let want_physics = toggles.get(Toggle::Physics);
    let physics = gizmos.config_mut::<PhysicsGizmos>().0;
    if physics.enabled != want_physics {
        physics.enabled = want_physics;
    }
}
