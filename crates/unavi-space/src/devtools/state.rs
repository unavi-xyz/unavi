use std::fmt::Write;

use bevy::{
    ecs::spawn::Spawn,
    feathers::{
        controls::{
            ButtonProps,
            ButtonVariant,
            button,
        },
        theme::ThemedText,
    },
    prelude::*,
    ui_widgets::Activate,
};
use unavi_devtools::tabs::DevPanel;

use crate::{
    devtools::{
        conn,
        short,
    },
    peer::self_peer_id,
    state::replicas::debug,
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
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
            ));
            p.spawn((
                Node {
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
            ))
            .with_children(|c| {
                c.spawn((
                    StateText,
                    Text::new("No peer state."),
                    ThemedText,
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                ));
            });
        });
}

/// Every connected peer plus self, sorted by id and flagged when self, so the
/// selector lists peers we have a connection to even before any state syncs.
fn peer_ids() -> Vec<([u8; 32], bool)> {
    let me = self_peer_id();
    let mut ids = me.into_iter().collect::<Vec<_>>();
    ids.extend(conn::snapshot().into_iter().map(|s| s.peer));
    ids.extend(debug::snapshot().into_iter().map(|p| p.peer));
    ids.sort_unstable();
    ids.dedup();
    ids.into_iter().map(|id| (id, Some(id) == me)).collect()
}

pub(super) fn sync_selector(
    mut stored: ResMut<SelectorPeers>,
    rows: Query<Entity, With<Selector>>,
    buttons: Query<Entity, With<SelectButton>>,
    mut commands: Commands,
) {
    let peers = peer_ids();
    let ids = peers.iter().map(|(p, _)| *p).collect::<Vec<_>>();
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
        for (peer, is_self) in &peers {
            let label = format!("{}{}", short(peer), if *is_self { " (self)" } else { "" });
            row.spawn(button(
                ButtonProps::default(),
                SelectButton(*peer),
                Spawn((Text::new(label), ThemedText)),
            ));
        }
    });
}

pub(super) fn handle_select(
    activate: On<Activate>,
    buttons: Query<&SelectButton>,
    mut selection: ResMut<StateSelection>,
) {
    if let Ok(button) = buttons.get(activate.entity) {
        selection.0 = Some(button.0);
    }
}

/// Paints the inspected peer's button in the primary variant, following the
/// local peer while the selection is unset.
pub(super) fn highlight_selected(
    selection: Res<StateSelection>,
    added: Query<(), Added<SelectButton>>,
    mut buttons: Query<(&SelectButton, &mut ButtonVariant)>,
) {
    if !selection.is_changed() && added.is_empty() {
        return;
    }
    let chosen = selection.0.or_else(self_peer_id);
    for (button, mut variant) in &mut buttons {
        *variant = if Some(button.0) == chosen {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Normal
        };
    }
}

pub(super) fn render(selection: Res<StateSelection>, mut text: Query<&mut Text, With<StateText>>) {
    let me = self_peer_id();
    let states = debug::snapshot();

    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let Some(chosen) = selection.0.or(me) else {
        text.0 = "No peer state.".into();
        return;
    };

    let mut out = format!(
        "Peer {}{}\n",
        short(&chosen),
        if Some(chosen) == me { " (self)" } else { "" }
    );
    let docs = states
        .iter()
        .find(|p| p.peer == chosen)
        .map(|p| p.docs.as_slice())
        .unwrap_or_default();
    if docs.is_empty() {
        let _ = writeln!(out, "{:2}(no docs)", "");
    }
    for d in docs {
        let _ = writeln!(
            out,
            "{:2}doc {} space {}{}{}",
            "",
            short(d.doc.as_bytes()),
            short(d.space.as_bytes()),
            d.pin.map(|p| format!(" [pin={p}]")).unwrap_or_default(),
            d.authority
                .map(|a| format!(" authority={a}"))
                .unwrap_or_default(),
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
