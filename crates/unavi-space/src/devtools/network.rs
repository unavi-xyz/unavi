use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use unavi_devtools::tabs::DevPanel;

use crate::devtools::{
    conn,
    short,
};

#[derive(Component)]
pub(super) struct NetworkPanel;

#[derive(Component)]
pub(super) struct NetworkText;

struct Prev {
    tx: u64,
    rx: u64,
    at: f32,
}

#[derive(Resource, Default)]
pub(super) struct NetSampler {
    prev: HashMap<[u8; 32], Prev>,
}

pub(super) fn spawn(mut commands: Commands) {
    commands
        .spawn((
            DevPanel {
                title: "Network".into(),
            },
            NetworkPanel,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                NetworkText,
                Text::new("No peers connected."),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.9, 0.95)),
            ));
        });
}

pub(super) fn update(
    time: Res<Time>,
    mut sampler: ResMut<NetSampler>,
    mut text: Query<&mut Text, With<NetworkText>>,
) {
    let now = time.elapsed_secs();
    let snap = conn::snapshot();

    let mut next = HashMap::new();
    let mut lines = Vec::new();
    let mut total_up = 0.0;
    let mut total_down = 0.0;

    for s in &snap {
        let (up, down) = match sampler.prev.get(&s.peer) {
            Some(p) if now > p.at => {
                let dt = now - p.at;
                (
                    s.bytes_tx.saturating_sub(p.tx) as f32 / dt,
                    s.bytes_rx.saturating_sub(p.rx) as f32 / dt,
                )
            }
            _ => (0.0, 0.0),
        };
        total_up += up;
        total_down += down;

        lines.push(format!(
            "{} ↑{:>7.1} ↓{:>7.1} KB/s  rtt {:>4.0}ms",
            short(&s.peer),
            up / 1024.0,
            down / 1024.0,
            s.rtt_ms,
        ));

        next.insert(
            s.peer,
            Prev {
                tx: s.bytes_tx,
                rx: s.bytes_rx,
                at: now,
            },
        );
    }
    sampler.prev = next;

    let Ok(mut text) = text.single_mut() else {
        return;
    };
    if snap.is_empty() {
        text.0 = "No peers connected.".into();
        return;
    }
    let mut out = format!(
        "Peers: {} TOTAL ↑{:.1} ↓{:.1} KB/s\n",
        snap.len(),
        total_up / 1024.0,
        total_down / 1024.0,
    );
    out.push_str(&lines.join("\n"));
    text.0 = out;
}
