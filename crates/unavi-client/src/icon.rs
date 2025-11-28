use anyhow::Result;
use bevy::{prelude::*, winit::WINIT_WINDOWS};
use winit::window::Icon;

const ICON_BYTES: &[u8] = include_bytes!("../assets/images/unavi-rounded.png");

/// Worlld param forces system to run on main thread, which is needed for `WINIT_WINDOWS` static.
pub fn set_window_icon(_world: &mut World) {
    match try_get_icon() {
        Ok(icon) => WINIT_WINDOWS.with_borrow(|windows| {
            if windows.windows.is_empty() {
                warn!("No windows found to set icon");
                return;
            }
            for window in windows.windows.values() {
                window.set_window_icon(Some(icon.clone()));
            }
        }),
        Err(e) => error!("Failed to get icon: {e:?}"),
    }
}

fn try_get_icon() -> Result<Icon> {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(ICON_BYTES)?.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height)?;
    Ok(icon)
}
