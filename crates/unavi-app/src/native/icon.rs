use std::path::PathBuf;

use anyhow::Result;
use bevy::{prelude::*, winit::WinitWindows};
use winit::window::Icon;

pub fn set_window_icon(windows: NonSend<WinitWindows>) {
    if let Ok(icon) = try_get_icon() {
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    }
}

/// Try to get the icon from the assets directory.
/// Will likely fail to find the file during devlopment, but
/// should work correctly in the distributed release.
fn try_get_icon() -> Result<Icon> {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(
            std::env::current_exe()?
                .parent()
                .unwrap()
                .join(PathBuf::from_iter(["assets", "images", "logo.png"])),
        )?
        .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height)?;
    Ok(icon)
}
