use avian3d::prelude::PhysicsGizmos;
use bevy::{
    dev_tools::fps_overlay::{
        FPS_OVERLAY_ZINDEX,
        FpsOverlayConfig,
    },
    feathers::controls::FeathersToggleSwitch,
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

pub(super) fn toggled(toggle: Toggle) -> impl Fn(Res<DevToggles>) -> bool + Clone {
    move |toggles: Res<DevToggles>| toggles.get(toggle)
}

pub(super) fn spawn(mut commands: Commands) {
    let panel = commands
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
        .id();

    for (label, toggle) in [
        ("FPS overlay", Toggle::Fps),
        ("World inspector", Toggle::Inspector),
        ("Physics gizmos", Toggle::Physics),
        ("Event gizmos", Toggle::Events),
    ] {
        let row = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                ChildOf(panel),
            ))
            .id();

        commands.spawn((
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            TextColor(Color::WHITE),
            ChildOf(row),
        ));

        // FeathersToggleSwitch is a scene component and must be spawned as a
        // scene.
        commands
            .spawn_scene(FeathersToggleSwitch::scene(()))
            .insert((toggle, ChildOf(row)))
            .observe(checkbox_self_update);
    }
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

/// The overlay's own root is hidden here rather than left to
/// [`FpsOverlayConfig`]: Bevy's toggle reads the frame time graph node, which
/// no `WebGL` build spawns, so on the web it never runs at all.
pub(super) fn apply_fps_display(
    toggles: Res<DevToggles>,
    mut nodes: Query<(&mut Node, &GlobalZIndex)>,
) {
    let display = if toggles.get(Toggle::Fps) {
        Display::DEFAULT
    } else {
        Display::None
    };

    for (mut node, depth) in &mut nodes {
        if depth.0 == FPS_OVERLAY_ZINDEX && node.display != display {
            node.display = display;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn overlay_display(fps_on: bool) -> Display {
        let mut app = App::new();
        app.insert_resource(DevToggles([fps_on, false, false, false]))
            .add_systems(Update, apply_fps_display);

        let root = app
            .world_mut()
            .spawn((Node::default(), GlobalZIndex(FPS_OVERLAY_ZINDEX)))
            .id();
        app.update();

        app.world()
            .entity(root)
            .get::<Node>()
            .expect("node")
            .display
    }

    #[test]
    fn the_fps_overlay_is_hidden_without_the_frame_time_graph_bevy_toggles_it_by() {
        assert_eq!(overlay_display(false), Display::None);
        assert_eq!(overlay_display(true), Display::DEFAULT);
    }
}
