use bevy::{
    color::Alpha,
    prelude::*,
};

use crate::style;

/// How much of the world shows through the body.
const BODY_ALPHA: f32 = 0.85;

/// Whether the overlay is currently open. Other crates read this to suppress
/// gameplay input while dev tools have focus.
#[derive(Resource, Default)]
pub struct DevToolsActive(pub(crate) bool);

impl DevToolsActive {
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.0
    }
}

/// Handles to the overlay's structural nodes, used when mounting panels.
#[derive(Resource)]
pub(crate) struct DevOverlay {
    pub root:    Entity,
    pub tab_bar: Entity,
    pub body:    Entity,
}

#[derive(Component)]
pub(crate) struct OverlayRoot;

pub(crate) fn spawn_overlay(mut commands: Commands) {
    let tab_bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(style::PANEL.into()),
            BorderColor::all(style::RULE),
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
            BackgroundColor(Color::from(style::BG).with_alpha(BODY_ALPHA)),
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

pub(crate) fn toggle_overlay(
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
