use std::fmt::Write;

use bevy::{
    ecs::{
        relationship::RelatedSpawnerCommands,
        spawn::Spawn,
    },
    feathers::{
        controls::ButtonBundleProps,
        theme::ThemedText,
    },
    prelude::*,
};

use crate::{
    devtools::{
        inspect::{
            LinkTo,
            Page,
        },
        short,
    },
    state::clock,
};

const VALUE_PREVIEW_MAX: usize = 256;

/// Millisecond timestamp re-rendered as a live relative time.
#[derive(Component)]
pub struct AgoText(pub u64);

pub fn refresh_times(mut texts: Query<(&AgoText, &mut Text)>) {
    for (at, mut text) in &mut texts {
        text.0 = fmt_ago(at.0);
    }
}

/// A stable per-id color derived from the leading hash bytes, so the same doc,
/// space, or peer is recognizable across pages.
pub fn hash_color(bytes: &[u8]) -> Color {
    let hue = f32::from(u16::from_be_bytes([bytes[0], bytes[1]])) / 65535.0 * 360.0;
    Color::hsl(hue, 0.6, 0.65)
}

pub fn swatch(color: Color) -> impl Bundle {
    (
        Node {
            width: Val::Px(8.0),
            height: Val::Px(8.0),
            flex_shrink: 0.0,
            border_radius: BorderRadius::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(color),
    )
}

/// Spawns a clickable id chip: colored swatch plus short hex, navigating to
/// `page`. The compact layout is inserted after spawn because the feathers
/// button bundle already carries a `Node` and duplicates panic.
#[expect(
    deprecated,
    reason = "feathers button() BSN requires scene spawning; button_bundle is the transitional API"
)]
pub fn chip(parent: &mut RelatedSpawnerCommands<ChildOf>, bytes: &[u8], page: Page) {
    let color = hash_color(bytes);
    parent
        .spawn(bevy::feathers::controls::button_bundle(
            ButtonBundleProps::default(),
            LinkTo(page),
            (
                Spawn(swatch(color)),
                Spawn((
                    Text::new(short(bytes)),
                    TextFont {
                        font_size: FontSize::Px(12.0),
                        ..default()
                    },
                    TextColor(color),
                )),
            ),
        ))
        .insert(Node {
            padding: UiRect::axes(Val::Px(5.0), Val::Px(1.0)),
            column_gap: Val::Px(4.0),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        });
}

/// Spawns a compact labeled button carrying `marker` (back or view toggles).
#[expect(
    deprecated,
    reason = "feathers button() BSN requires scene spawning; button_bundle is the transitional API"
)]
pub fn small_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    marker: impl Bundle,
) {
    parent
        .spawn(bevy::feathers::controls::button_bundle(
            ButtonBundleProps::default(),
            marker,
            Spawn((
                Text::new(label),
                ThemedText,
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
            )),
        ))
        .insert(Node {
            padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(Val::Px(3.0)),
            ..default()
        });
}

pub fn row_node() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(6.0),
        row_gap: Val::Px(4.0),
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        ..default()
    }
}

pub fn grid_node(cols: usize) -> Node {
    Node {
        display: Display::Grid,
        grid_template_columns: vec![GridTrack::auto(); cols],
        column_gap: Val::Px(14.0),
        row_gap: Val::Px(3.0),
        justify_items: JustifyItems::Start,
        align_items: AlignItems::Center,
        ..default()
    }
}

pub fn section_title(title: &str) -> impl Bundle {
    (
        Text::new(title),
        TextFont {
            font_size: FontSize::Px(13.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
        Node {
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
        },
    )
}

pub fn header_cell(header: &str) -> impl Bundle {
    (
        Text::new(header.to_uppercase()),
        TextFont {
            font_size: FontSize::Px(10.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
    )
}

pub fn value_text(value: String) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
    )
}

pub fn dim_text(value: &str) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
    )
}

pub fn mono_block(content: String) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        children![(
            Text::new(content),
            TextFont {
                font_size: FontSize::Px(11.0),
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.85, 0.75)),
        )],
    )
}

/// A KV value expanded under its row, spanning `cols` grid columns: a hex dump
/// plus the lossy text form, both capped.
pub fn value_detail(value: &[u8], cols: usize) -> impl Bundle {
    let shown = &value[..value.len().min(VALUE_PREVIEW_MAX)];
    let mut hex = shown
        .chunks(16)
        .map(|row| {
            row.iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    if value.len() > VALUE_PREVIEW_MAX {
        let _ = write!(hex, "\n… +{} more bytes", value.len() - VALUE_PREVIEW_MAX);
    }
    let text = format!("text: {}", String::from_utf8_lossy(shown));
    (
        Node {
            grid_column: GridPlacement::span(u16::try_from(cols).unwrap_or(1)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        children![
            (
                Text::new(hex),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.85, 0.75)),
            ),
            (
                Text::new(text),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
            ),
        ],
    )
}

pub fn fmt_ago(at: u64) -> String {
    let now = clock::current_millis();
    let (secs, suffix) = if now >= at {
        ((now - at) / 1000, "ago")
    } else {
        ((at - now) / 1000, "ahead")
    };
    if secs < 60 {
        format!("{secs}s {suffix}")
    } else if secs < 3600 {
        format!("{}m {}s {suffix}", secs / 60, secs % 60)
    } else {
        format!("{}h {}m {suffix}", secs / 3600, (secs % 3600) / 60)
    }
}

pub fn fmt_size(len: usize) -> String {
    if len < 1024 {
        format!("{len} B")
    } else {
        format!("{}.{} KB", len / 1024, (len % 1024) * 10 / 1024)
    }
}
