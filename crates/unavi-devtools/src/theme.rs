//! The overlay's feathers theme, painted from the design tokens.

use bevy::{
    color::Color,
    feathers::{
        dark_theme::create_dark_theme,
        theme::ThemeProps,
        tokens::{
            self,
            semantic,
        },
    },
};

use crate::style;

/// Feathers' own control wiring, repainted light: `bg` and `panel` grounds,
/// `fg` type, `rule` hairlines, and `blue` for the one active thing.
#[must_use]
pub fn light_theme() -> ThemeProps {
    let [bg, panel, tint, rule, fg, muted, blue, light] = [
        style::BG,
        style::PANEL,
        style::TINT,
        style::RULE,
        style::FG,
        style::MUTED,
        style::BLUE,
        style::LIGHT,
    ]
    .map(Color::from);

    let mut theme = create_dark_theme();
    theme.semantic_overrides.clear();
    theme.semantic_base.extend([
        (semantic::SURFACE_WINDOW, bg),
        (semantic::SURFACE_PANE_BODY, panel),
        (semantic::SURFACE_PANE_HEADER, bg),
        (semantic::SURFACE_SUBPANE_BODY, panel),
        (semantic::SURFACE_SUBPANE_HEADER, tint),
        (semantic::SURFACE_GROUP, tint),
        (semantic::SURFACE_DIALOG, panel),
        (semantic::SURFACE_MENU, panel),
        (semantic::BORDER_DEFAULT, rule),
        (semantic::TEXT_DEFAULT, fg),
        (semantic::TEXT_DIM, muted),
        (semantic::TEXT_DISABLED, muted),
        (semantic::TEXT_ON_ACCENT, light),
        (semantic::FILL_ACCENT_DEFAULT, blue),
        (semantic::FILL_ACCENT_HOVER, blue),
        (semantic::FILL_ACCENT_PRESSED, fg),
        (semantic::FILL_ACCENT_DISABLED, rule),
        (semantic::FILL_SOLID_DEFAULT, tint),
        (semantic::FILL_SOLID_HOVER, rule),
        (semantic::FILL_SOLID_PRESSED, rule),
        (semantic::FILL_SOLID_DISABLED, tint),
        (semantic::FILL_ITEM_HOVER, tint),
        (semantic::FILL_ITEM_PRESSED, rule),
        (semantic::FILL_FIELD_DEFAULT, panel),
        (semantic::FILL_FIELD_HOVER, tint),
        (semantic::FILL_FIELD_PRESSED, tint),
        (semantic::FOCUS_RING, blue),
    ]);

    // These sit on a light fill here, where the dark theme puts light type on
    // a dark one.
    for token in [
        tokens::BUTTON_TEXT,
        tokens::MENUITEM_TEXT,
        tokens::LISTROW_TEXT,
        tokens::TEXT_INPUT_TEXT,
    ] {
        theme
            .token_assignments
            .insert(token, semantic::TEXT_DEFAULT);
    }
    theme
}
