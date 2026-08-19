use bevy::{
    prelude::*,
    ui::Val,
};

use crate::{
    capture::Captured,
    pointer::{
        PointerAnchor,
        PointerKind,
    },
};

const SIZE: f32 = 5.0;
/// The ring drawn around the dot when something is worth taking hold of.
const RING_SIZE: f32 = 16.0;
const RING_WIDTH: f32 = 1.5;

const RESTING: Color = Color::srgba(1.0, 1.0, 1.0, 0.55);
const OVER_GRABBABLE: Color = Color::srgba(1.0, 1.0, 1.0, 0.9);

#[derive(Component)]
#[require(CrosshairMode)]
pub struct Crosshair;

#[derive(Component)]
pub(crate) struct CrosshairRing;

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
pub enum CrosshairMode {
    Active,
    #[default]
    Inactive,
}

/// A fixed mark at the centre of the screen rather than a reticle laid on the
/// surface underfoot: the screen pointer always aims at the middle of the
/// window, so the mark belongs there too and needs no world position to be
/// read correctly.
pub(crate) fn spawn_crosshair(mut commands: Commands) {
    commands
        .spawn((
            Crosshair,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            // The mark is not a thing to press, and a full-screen node over
            // everything would otherwise swallow whatever is behind it.
            Pickable::IGNORE,
            GlobalZIndex(i32::MAX),
            Visibility::Hidden,
        ))
        .with_children(|screen| {
            screen.spawn((
                CrosshairRing,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(RING_SIZE),
                    height: Val::Px(RING_SIZE),
                    border: UiRect::all(Val::Px(RING_WIDTH)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BorderColor::all(OVER_GRABBABLE),
                Pickable::IGNORE,
                Visibility::Hidden,
            ));
            screen.spawn((
                Node {
                    width: Val::Px(SIZE),
                    height: Val::Px(SIZE),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(RESTING),
                Pickable::IGNORE,
            ));
        });
}

/// Shows the mark whenever there is a screen pointer to mark, which is every
/// frame on desktop and none in VR, where the hands do their own aiming.
///
/// Something drawn over the world takes it with the rest of the input: a mark
/// aiming at what is behind an overlay is a mark aiming at nothing.
pub(crate) fn show_crosshair(
    pointers: Query<&PointerAnchor>,
    captured: Res<Captured>,
    mut crosshair: Query<&mut Visibility, With<Crosshair>>,
) {
    let Ok(mut visibility) = crosshair.single_mut() else {
        return;
    };
    let on_screen = !captured.0
        && pointers
            .iter()
            .any(|anchor| anchor.0 == PointerKind::Screen);

    let wanted = if on_screen {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if *visibility != wanted {
        *visibility = wanted;
    }
}

/// The ring is the whole of the active state: a mark that changed shape would
/// move the eye, and what it is reporting is only that the grip has something
/// to take.
pub(crate) fn apply_crosshair_mode(
    crosshair: Query<&CrosshairMode, With<Crosshair>>,
    mut ring: Query<&mut Visibility, With<CrosshairRing>>,
) {
    let (Ok(mode), Ok(mut visibility)) = (crosshair.single(), ring.single_mut()) else {
        return;
    };

    let wanted = if *mode == CrosshairMode::Active {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if *visibility != wanted {
        *visibility = wanted;
    }
}
