//! Bevy UI overlay for network stats.

use bevy::prelude::*;

use super::network_stats::{ConnectionQuality, NetworkStats};

#[derive(Component)]
pub struct DevToolsOverlay;

#[derive(Component)]
pub struct NetworkStatsText {
    #[allow(dead_code)]
    pub host_url: String,
}

pub fn spawn_devtools_overlay(mut commands: Commands) {
    commands
        .spawn((
            DevToolsOverlay,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Network Debug"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });
}

pub fn update_network_stats_text(
    stats: Res<NetworkStats>,
    mut commands: Commands,
    overlay_query: Query<Entity, With<DevToolsOverlay>>,
    text_query: Query<(Entity, &NetworkStatsText)>,
) {
    if !stats.is_changed() && !stats.hosts.is_empty() {
        return;
    }

    let Ok(overlay_entity) = overlay_query.single() else {
        return;
    };

    // Remove old stat text entities.
    for (entity, _) in text_query.iter() {
        commands.entity(entity).despawn();
    }

    // Add new stat text for each host.
    for (host_url, host_stats) in &stats.hosts {
        let upload_kb = host_stats.upload_bytes_per_sec / 1024.0;
        let download_kb = host_stats.download_bytes_per_sec / 1024.0;

        #[allow(clippy::cast_sign_loss)]
        let iframe_pct = (host_stats.iframe_upload_ratio * 100.0) as u32;
        #[allow(clippy::cast_sign_loss)]
        let pframe_pct = (host_stats.pframe_upload_ratio * 100.0) as u32;

        let drop_pct = if host_stats.total_frames_received > 0 {
            (host_stats.dropped_frames as f32 / host_stats.total_frames_received as f32) * 100.0
        } else {
            0.0
        };

        let quality_str = match host_stats.quality_score {
            ConnectionQuality::Excellent => "Excellent",
            ConnectionQuality::Good => "Good",
            ConnectionQuality::Fair => "Fair",
            ConnectionQuality::Poor => "Poor",
        };

        let quality_color = match host_stats.quality_score {
            ConnectionQuality::Excellent => Color::srgb(0.0, 1.0, 0.0),
            ConnectionQuality::Good => Color::srgb(0.5, 1.0, 0.0),
            ConnectionQuality::Fair => Color::srgb(1.0, 0.7, 0.0),
            ConnectionQuality::Poor => Color::srgb(1.0, 0.0, 0.0),
        };

        let stats_text = format!(
            "Host: {}\n\
             ↑ Upload:   {:.1} KB/s (I: {}%, P: {}%)\n\
             ↓ Download: {:.1} KB/s\n\
             Tickrate:   {:.1} Hz\n\
             Dropped:    {} ({:.1}%)\n\
             Latency:    ~{:.0}ms\n\
             Quality:    {}",
            shorten_url(host_url),
            upload_kb,
            iframe_pct,
            pframe_pct,
            download_kb,
            host_stats.effective_tickrate,
            host_stats.dropped_frames,
            drop_pct,
            host_stats.estimated_latency_ms,
            quality_str
        );

        commands.entity(overlay_entity).with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                    NetworkStatsText {
                        host_url: host_url.clone(),
                    },
                ))
                .with_children(|parent| {
                    for (i, line) in stats_text.lines().enumerate() {
                        let color = if line.contains("Quality:") {
                            quality_color
                        } else if i == 0 {
                            Color::srgb(0.7, 0.8, 1.0)
                        } else {
                            Color::srgb(0.85, 0.85, 0.85)
                        };

                        parent.spawn((
                            Text::new(line),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                });
        });
    }
}

fn shorten_url(url: &str) -> &str {
    // Extract just the host part for cleaner display.
    url.split("://")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or(url)
}
