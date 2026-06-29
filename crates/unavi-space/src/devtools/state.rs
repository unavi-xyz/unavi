use std::fmt::Write;

use bevy::prelude::*;
use unavi_devtools::tabs::DevPanel;

use crate::{
    devtools::short,
    state::peer::debug,
};

#[derive(Component)]
pub(super) struct StatePanel;

#[derive(Component)]
pub(super) struct Selector;

#[derive(Component)]
pub(super) struct StateText;

#[derive(Component)]
pub(super) struct SelectButton([u8; 32]);

/// The inspected peer, or `None` to follow the local peer.
#[derive(Resource, Default)]
pub(super) struct StateSelection(Option<[u8; 32]>);

/// Peer ids the selector row currently shows, so it rebuilds only on change.
#[derive(Resource, Default)]
pub(super) struct SelectorPeers(Vec<[u8; 32]>);

pub(super) fn spawn(mut commands: Commands) {
    commands
        .spawn((
            DevPanel {
                title: "Peer State".into(),
            },
            StatePanel,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                Selector,
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
            ));
            p.spawn((
                StateText,
                Text::new("No peer state."),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.9, 0.95)),
            ));
        });
}

pub(super) fn sync_selector(
    mut stored: ResMut<SelectorPeers>,
    rows: Query<Entity, With<Selector>>,
    buttons: Query<Entity, With<SelectButton>>,
    mut commands: Commands,
) {
    let peers = debug::snapshot();
    let mut ids = peers.iter().map(|p| p.peer).collect::<Vec<_>>();
    ids.sort_unstable();
    if ids == stored.0 {
        return;
    }
    stored.0 = ids;

    for button in &buttons {
        commands.entity(button).despawn();
    }
    let Ok(row) = rows.single() else {
        return;
    };
    commands.entity(row).with_children(|row| {
        for p in &peers {
            let label = format!("{}{}", short(&p.peer), if p.is_self { "*" } else { "" });
            row.spawn((
                Button,
                SelectButton(p.peer),
                Node {
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                children![(
                    Text::new(label),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )],
            ));
        }
    });
}

pub(super) fn handle_select(
    interactions: Query<(&Interaction, &SelectButton), Changed<Interaction>>,
    mut selection: ResMut<StateSelection>,
) {
    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed {
            selection.0 = Some(button.0);
        }
    }
}

pub(super) fn render(selection: Res<StateSelection>, mut text: Query<&mut Text, With<StateText>>) {
    let peers = debug::snapshot();
    let chosen = selection
        .0
        .or_else(|| peers.iter().find(|p| p.is_self).map(|p| p.peer));

    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let Some(p) = chosen.and_then(|c| peers.iter().find(|p| p.peer == c)) else {
        text.0 = "No peer state.".into();
        return;
    };

    let mut out = format!(
        "Peer {}{}\n",
        short(&p.peer),
        if p.is_self { " (self)" } else { "" }
    );
    if p.docs.is_empty() {
        let _ = writeln!(out, "{:2}(no docs)", "");
    }
    for d in &p.docs {
        let _ = writeln!(
            out,
            "{:2}doc {} space {}{}{}",
            "",
            short(d.doc.as_bytes()),
            short(d.space.as_bytes()),
            if d.pinned { " [pinned]" } else { "" },
            d.claim.map(|c| format!(" claim={c}")).unwrap_or_default(),
        );
        for kv in &d.kv {
            let _ = match kv.bytes {
                Some(n) => writeln!(out, "{:4}{} = {n} bytes", "", kv.key),
                None => writeln!(out, "{:4}{} = <tombstone>", "", kv.key),
            };
        }
    }
    text.0 = out;
}
