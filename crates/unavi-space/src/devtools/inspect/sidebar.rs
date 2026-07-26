use bevy::{
    ecs::{
        relationship::RelatedSpawnerCommands,
        spawn::Spawn,
    },
    feathers::controls::{
        ButtonProps,
        ButtonVariant,
        button,
    },
    prelude::*,
};
use blake3::Hash;

use crate::{
    Space,
    anchor::ActiveSpace,
    devtools::{
        conn,
        inspect::{
            CurrentPage,
            LinkTo,
            Page,
            effective_page,
            model,
            widgets,
        },
        short,
    },
    peer::self_peer_id,
    state::replicas::debug,
};

/// The scrollable navigation list: spaces on top, then every known peer.
#[derive(Component)]
pub struct SidebarList;

/// A sidebar entry navigating to its page; only these highlight as selected.
#[derive(Component)]
pub struct SidebarButton(Page);

#[derive(Clone, PartialEq, Eq)]
pub enum SidebarRow {
    Header(&'static str),
    Entry { page: Page, label: String },
}

#[derive(Resource, Default)]
pub struct SidebarEntries(Vec<SidebarRow>);

fn rows(spaces: &Query<(Entity, &Space)>, active: Option<Hash>) -> Vec<SidebarRow> {
    let snap = debug::snapshot();
    let me = self_peer_id();
    let mut out = Vec::new();

    let mut space_ids = spaces.iter().map(|(_, s)| s.0).collect::<Vec<_>>();
    space_ids.extend(
        snap.peers
            .iter()
            .flat_map(|p| p.docs.iter())
            .chain(snap.neutral.iter())
            .map(|d| d.space),
    );
    space_ids.sort_unstable_by_key(|s| *s.as_bytes());
    space_ids.dedup();
    if !space_ids.is_empty() {
        out.push(SidebarRow::Header("Spaces"));
        if let Some(active) = active {
            space_ids.retain(|s| *s != active);
            out.push(SidebarRow::Entry {
                page:  Page::Space(active),
                label: format!("{} (active)", short(active.as_bytes())),
            });
        }
        for space in space_ids {
            out.push(SidebarRow::Entry {
                page:  Page::Space(space),
                label: short(space.as_bytes()),
            });
        }
    }

    let mut peer_ids = conn::snapshot().iter().map(|s| s.peer).collect::<Vec<_>>();
    peer_ids.extend(snap.peers.iter().map(|p| p.peer));
    peer_ids.sort_unstable();
    peer_ids.dedup();
    out.push(SidebarRow::Header("Peers"));
    if let Some(me) = me {
        peer_ids.retain(|p| *p != me);
        out.push(SidebarRow::Entry {
            page:  Page::Peer(me),
            label: format!("{} (self)", short(&me)),
        });
    }
    for peer in peer_ids {
        out.push(SidebarRow::Entry {
            page:  Page::Peer(peer),
            label: short(&peer),
        });
    }
    out
}

fn entry_button(l: &mut RelatedSpawnerCommands<ChildOf>, page: Page, label: String) {
    let bytes = match page {
        Page::Peer(id) => id,
        Page::Space(hash) | Page::Doc(hash) => *hash.as_bytes(),
    };
    let color = widgets::hash_color(&bytes);
    l.spawn(button(
        ButtonProps::default(),
        (SidebarButton(page), LinkTo(page)),
        (
            Spawn(widgets::swatch(color)),
            Spawn((
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(color),
            )),
        ),
    ))
    .insert(Node {
        width: Val::Percent(100.0),
        padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
        column_gap: Val::Px(5.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::FlexStart,
        flex_shrink: 0.0,
        border_radius: BorderRadius::all(Val::Px(3.0)),
        ..default()
    });
}

pub fn sync(
    spaces: Query<(Entity, &Space)>,
    active: Res<ActiveSpace>,
    mut stored: ResMut<SidebarEntries>,
    list: Query<Entity, With<SidebarList>>,
    mut commands: Commands,
) {
    let rows = rows(&spaces, model::active_space(&spaces, active.0));
    if rows == stored.0 {
        return;
    }
    stored.0.clone_from(&rows);

    let Ok(list) = list.single() else {
        return;
    };
    commands.entity(list).despawn_related::<Children>();
    commands.entity(list).with_children(|l| {
        for row in rows {
            match row {
                SidebarRow::Header(header) => {
                    l.spawn((
                        widgets::header_cell(header),
                        Node {
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                    ));
                }
                SidebarRow::Entry { page, label } => entry_button(l, page, label),
            }
        }
    });
}

/// Paints the entry matching the current page in the primary variant.
pub fn highlight(
    current: Res<CurrentPage>,
    added: Query<(), Added<SidebarButton>>,
    mut buttons: Query<(&SidebarButton, &mut ButtonVariant)>,
) {
    if !current.is_changed() && added.is_empty() {
        return;
    }
    let chosen = effective_page(&current);
    for (button, mut variant) in &mut buttons {
        *variant = if Some(button.0) == chosen {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Normal
        };
    }
}
