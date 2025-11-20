use anyhow::Result;
use bevy::winit::WINIT_WINDOWS;
use winit::window::Icon;

const ICON_BYTES: &[u8] = include_bytes!("../../assets/images/unavi-rounded.png");

pub fn set_window_icon() {
    if let Ok(icon) = try_get_icon() {
        WINIT_WINDOWS.with_borrow(|windows| {
            for window in windows.windows.values() {
                window.set_window_icon(Some(icon.clone()));
            }
        });
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
