use bevy::{
    feathers::{
        constants::{
            fonts,
            size,
        },
        controls::ButtonVariant,
        font_styles::InheritableFont,
        theme::ThemedText,
    },
    prelude::*,
    text::FontWeight,
    ui_widgets::Activate,
};

use crate::{
    button::button,
    overlay::DevOverlay,
};

/// Marks a node as a dev tools panel's content root. Spawn one (with a panel
/// marker component) to contribute a tab; the framework mounts it and builds
/// the tab button.
#[derive(Component)]
pub struct DevPanel {
    pub title: String,
}

/// Present on the panel whose tab is currently selected.
///
/// Panel systems filter on this so their work only runs while visible; it
/// exists alongside [`ActiveDevPanel`] because [`panel_active`]'s query needs
/// a queryable marker, not an entity to compare against. `pub` because
/// `panel_active`'s query type names it.
#[derive(Component)]
pub struct ActivePanel;

#[derive(Component)]
pub(crate) struct PanelButton(Entity);

/// The one panel [`ActivePanel`] is mirroring. Kept so systems that need the
/// entity itself, not just "is mine active", do not have to query for it.
#[derive(Resource, Default)]
pub(crate) struct ActiveDevPanel(pub Option<Entity>);

/// Run condition: true only while the panel marked `M` is the active tab.
#[must_use]
pub fn panel_active<M: Component>(q: Query<(), (With<M>, With<ActivePanel>)>) -> bool {
    !q.is_empty()
}

pub(crate) fn register_panel(
    add: On<Add<DevPanel>>,
    panels: Query<&DevPanel>,
    overlay: Option<Res<DevOverlay>>,
    assets: Res<AssetServer>,
    mut active: ResMut<ActiveDevPanel>,
    mut commands: Commands,
) {
    let Some(overlay) = overlay else {
        return;
    };
    let panel = add.entity;
    let Ok(dev_panel) = panels.get(panel) else {
        return;
    };
    let title = dev_panel.title.clone();

    let font = assets.load(fonts::REGULAR);
    commands.entity(panel).insert(ChildOf(overlay.body));
    commands.entity(overlay.tab_bar).with_children(|tabs| {
        tabs.spawn(button(
            Node {
                height: size::ROW_HEIGHT,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(px(8)),
                ..default()
            },
            (
                PanelButton(panel),
                InheritableFont {
                    font,
                    font_size: size::MEDIUM_FONT,
                    weight: FontWeight::NORMAL,
                },
            ),
            Spawn((Text::new(title), ThemedText)),
        ));
    });

    if active.0.is_none() {
        active.0 = Some(panel);
    }
}

pub(crate) fn activate_panel(
    act: On<Activate>,
    buttons: Query<&PanelButton>,
    mut active: ResMut<ActiveDevPanel>,
) {
    if let Ok(btn) = buttons.get(act.entity) {
        active.0 = Some(btn.0);
    }
}

pub(crate) fn highlight_active_tab(
    active: Res<ActiveDevPanel>,
    added: Query<(), Added<PanelButton>>,
    mut buttons: Query<(&PanelButton, &mut ButtonVariant)>,
) {
    if !active.is_changed() && added.is_empty() {
        return;
    }
    for (button, mut variant) in &mut buttons {
        *variant = if active.0 == Some(button.0) {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Normal
        };
    }
}

pub(crate) fn apply_active_panel(
    active: Res<ActiveDevPanel>,
    added: Query<(), Added<DevPanel>>,
    mut panels: Query<(Entity, &mut Node, Has<ActivePanel>), With<DevPanel>>,
    mut commands: Commands,
) {
    if !active.is_changed() && added.is_empty() {
        return;
    }
    for (entity, mut node, has_active) in &mut panels {
        let is_active = active.0 == Some(entity);
        let want = if is_active {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != want {
            node.display = want;
        }
        if is_active && !has_active {
            commands.entity(entity).insert(ActivePanel);
        } else if !is_active && has_active {
            commands.entity(entity).remove::<ActivePanel>();
        }
    }
}
