use std::{
    collections::HashSet,
    hash::{
        DefaultHasher,
        Hash,
        Hasher,
    },
};

use bevy::{
    prelude::*,
    ui_widgets::Activate,
};
use iroh_docs::NamespaceId;
use unavi_devtools::{
    scroll::Scrollable,
    tabs::DevPanel,
};

use crate::{
    devtools::inspect::model::InspectData,
    peer::self_peer_id,
    state::replicas::debug,
};

pub mod model;
mod pages;
pub mod sidebar;
pub mod widgets;

const HISTORY_MAX: usize = 32;

/// A navigable inspector page. Id chips throughout the panel link between
/// pages.
#[derive(Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub enum Page {
    Peer([u8; 32]),
    Space(NamespaceId),
    Doc(NamespaceId),
}

#[derive(Component)]
pub struct InspectPanel;

/// Container whose children are rebuilt whenever the rendered page changes.
#[derive(Component)]
pub struct PageRoot;

/// Navigates to a page when its carrier is activated.
#[derive(Component)]
pub struct LinkTo(pub Page);

#[derive(Component)]
pub struct BackButton;

/// Ejects a peer, or lifts the block on one already ejected.
#[derive(Component)]
pub struct BlockButton {
    peer:    [u8; 32],
    blocked: bool,
}

/// Toggles a KV cell's value view open or closed.
#[derive(Component)]
pub struct ExpandButton {
    doc: NamespaceId,
    key: String,
}

/// The open page, or `None` to follow the local peer.
#[derive(Resource, Default)]
pub struct CurrentPage(Option<Page>);

#[derive(Resource, Default)]
pub struct History(Vec<Page>);

/// KV cells expanded into their value view, keyed by document and key.
#[derive(Resource, Default)]
pub struct Expanded(pub HashSet<(NamespaceId, String)>);

/// Fingerprint of the last built page, so the tree rebuilds only on change and
/// buttons stay stable under the pointer in between.
#[derive(Resource, Default)]
pub struct RenderedPage(u64);

pub fn effective_page(current: &CurrentPage) -> Option<Page> {
    current.0.or_else(|| self_peer_id().map(Page::Peer))
}

pub fn spawn(mut commands: Commands) {
    commands
        .spawn((
            DevPanel {
                title: "State".into(),
            },
            InspectPanel,
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                sidebar::SidebarList,
                Scrollable,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    width: Val::Px(170.0),
                    flex_shrink: 0.0,
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
            ));
            p.spawn((
                PageRoot,
                Scrollable,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: Val::Px(6.0),
                    flex_grow: 1.0,
                    min_width: Val::Px(0.0),
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
            ));
        });
}

pub fn handle_link(
    activate: On<Activate>,
    links: Query<&LinkTo>,
    mut current: ResMut<CurrentPage>,
    mut history: ResMut<History>,
) {
    let Ok(link) = links.get(activate.entity) else {
        return;
    };
    let from = effective_page(&current);
    if from == Some(link.0) {
        return;
    }
    current.0 = Some(link.0);
    if let Some(from) = from {
        if history.0.len() >= HISTORY_MAX {
            history.0.remove(0);
        }
        history.0.push(from);
    }
}

pub fn handle_back(
    activate: On<Activate>,
    back: Query<(), With<BackButton>>,
    mut current: ResMut<CurrentPage>,
    mut history: ResMut<History>,
) {
    if back.get(activate.entity).is_err() {
        return;
    }
    if let Some(page) = history.0.pop() {
        current.0 = Some(page);
    }
}

/// Blocks or unblocks the peer whose page is open — the only UI path that
/// reaches the trust table.
pub fn handle_block(activate: On<Activate>, buttons: Query<&BlockButton>) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };
    let result = if button.blocked {
        crate::trust::unblock(button.peer)
    } else {
        crate::trust::eject(button.peer)
    };
    if let Err(err) = result {
        warn!(?err, "cannot change a peer's rung");
    }
}

pub fn handle_expand(
    activate: On<Activate>,
    buttons: Query<&ExpandButton>,
    mut expanded: ResMut<Expanded>,
) {
    let Ok(button) = buttons.get(activate.entity) else {
        return;
    };
    let cell = (button.doc, button.key.clone());
    if !expanded.0.remove(&cell) {
        expanded.0.insert(cell);
    }
}

pub fn render(
    current: Res<CurrentPage>,
    history: Res<History>,
    expanded: Res<Expanded>,
    mut rendered: ResMut<RenderedPage>,
    data: InspectData,
    root: Query<Entity, With<PageRoot>>,
    mut commands: Commands,
) {
    let snap = debug::snapshot();
    let page = effective_page(&current);
    let model = page.map(|p| data.page_model(p, &snap));
    let can_back = !history.0.is_empty();

    let fp = fingerprint(page, model.as_ref(), &expanded, can_back);
    if rendered.0 == fp {
        return;
    }
    rendered.0 = fp;

    let Ok(root) = root.single() else {
        return;
    };
    commands.entity(root).despawn_related::<Children>();
    commands.entity(root).with_children(|b| {
        if let Some(model) = &model {
            pages::build(b, model, &expanded, can_back);
        } else {
            b.spawn(widgets::dim_text("No local peer."));
        }
    });
}

fn fingerprint(
    page: Option<Page>,
    model: Option<&model::PageModel>,
    expanded: &Expanded,
    can_back: bool,
) -> u64 {
    let mut h = DefaultHasher::new();
    page.hash(&mut h);
    model.hash(&mut h);
    can_back.hash(&mut h);
    let mut open = expanded.0.iter().collect::<Vec<_>>();
    open.sort_unstable_by(|a, b| {
        a.0.as_bytes()
            .cmp(b.0.as_bytes())
            .then_with(|| a.1.cmp(&b.1))
    });
    for (doc, key) in open {
        doc.as_bytes().hash(&mut h);
        key.hash(&mut h);
    }
    h.finish()
}
