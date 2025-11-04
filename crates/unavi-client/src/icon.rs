use anyhow::Result;
use bevy::{prelude::*, winit::WinitWindows};
use winit::window::Icon;

use crate::images_dir;

pub fn set_window_icon(windows: NonSend<WinitWindows>) {
    if let Ok(icon) = try_get_icon() {
        for window in windows.windows.values() {
            window.set_window_icon(Some(icon.clone()));
        }
    }
}

fn try_get_icon() -> Result<Icon> {
    let icon_path = images_dir().join("logo.png");

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(icon_path)?.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height)?;
    Ok(icon)
}
