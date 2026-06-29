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

use crate::overlay::DevOverlay;

/// Marks a node as a dev tools panel's content root. Spawn one (with a panel
/// marker component) to contribute a tab; the framework mounts it and builds the
/// tab button.
#[derive(Component)]
pub struct DevPanel {
    pub title: String,
}

/// Present on the panel whose tab is currently selected. Panel systems filter on
/// this so their work only runs while visible.
#[derive(Component)]
pub struct ActivePanel;

#[derive(Component)]
pub(crate) struct PanelButton(Entity);

#[derive(Resource, Default)]
pub struct ActiveDevPanel(pub Option<Entity>);

/// Run condition: true only while the panel marked `M` is the active tab.
#[must_use]
pub fn panel_active<M: Component>(q: Query<(), (With<M>, With<ActivePanel>)>) -> bool {
    !q.is_empty()
}

pub(crate) fn register_panel(
    add: On<Add, DevPanel>,
    panels: Query<&DevPanel>,
    overlay: Option<Res<DevOverlay>>,
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

    commands.entity(panel).insert(ChildOf(overlay.body));
    commands.entity(overlay.tab_bar).with_children(|tabs| {
        tabs.spawn(button(
            ButtonProps::default(),
            PanelButton(panel),
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
