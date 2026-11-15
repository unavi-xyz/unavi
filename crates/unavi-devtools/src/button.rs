//! A feathers button that carries the caller's own component and children.
//!
//! Feathers spawns its button as a scene component in 0.20, and a scene owns
//! its entity, so an application marker cannot ride along. This spells the
//! scene out as a plain bundle: same components, same styling systems.

use bevy::{
    ecs::spawn::SpawnableList,
    feathers::{
        controls::ButtonVariant,
        focus::FocusIndicator,
        theme::{
            InheritableThemeTextColor,
            ThemeBackgroundColor,
        },
        tokens,
    },
    input_focus::tab_navigation::TabIndex,
    picking::{
        cursor::EntityCursor,
        hover::Hovered,
    },
    prelude::*,
    ui_widgets::Button,
    window::SystemCursorIcon,
};

/// A feathers-styled button with `node` as its layout, `marker` attached to
/// the button itself, and `content` spawned as its children.
pub fn button<M, L>(node: Node, marker: M, content: L) -> impl Bundle
where
    M: Bundle,
    L: SpawnableList<ChildOf> + Send + Sync + 'static,
{
    (
        node,
        (
            Button,
            ButtonVariant::Normal,
            Hovered::default(),
            EntityCursor::System(SystemCursorIcon::Pointer),
        ),
        (TabIndex(0), FocusIndicator),
        (
            ThemeBackgroundColor(tokens::BUTTON_BG),
            InheritableThemeTextColor(tokens::BUTTON_TEXT),
        ),
        marker,
        Children::spawn(content),
    )
}
