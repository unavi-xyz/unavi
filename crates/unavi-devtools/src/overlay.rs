use bevy::prelude::*;

/// Whether the overlay is currently open. Other crates read this to suppress
/// gameplay input while dev tools have focus.
#[derive(Resource, Default)]
pub struct DevToolsActive(pub bool);

/// Handles to the overlay's structural nodes, used when mounting panels.
#[derive(Resource)]
pub struct DevOverlay {
    pub root:    Entity,
    pub tab_bar: Entity,
    pub body:    Entity,
}

#[derive(Component)]
pub struct OverlayRoot;

pub fn spawn_overlay(mut commands: Commands) {
    let tab_bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .id();

    let body = commands
        .spawn(Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            overflow: Overflow::clip(),
            ..default()
        })
        .id();

    let root = commands
        .spawn((
            OverlayRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.85)),
            GlobalZIndex(1000),
        ))
        .add_children(&[tab_bar, body])
        .id();

    commands.insert_resource(DevOverlay {
        root,
        tab_bar,
        body,
    });
}

pub fn toggle_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    overlay: Option<Res<DevOverlay>>,
    mut active: ResMut<DevToolsActive>,
    mut roots: Query<&mut Node, With<OverlayRoot>>,
) {
    if !keys.just_pressed(KeyCode::Backquote) {
        return;
    }
    let Some(overlay) = overlay else {
        return;
    };
    active.0 = !active.0;
    if let Ok(mut node) = roots.get_mut(overlay.root) {
        node.display = if active.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}
