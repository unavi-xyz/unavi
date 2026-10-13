use anyhow::Result;
use bevy::{
    prelude::*,
    winit::WINIT_WINDOWS,
};
use winit::window::Icon;

const ICON_BYTES: &[u8] = include_bytes!("../../../assets/icon-rounded.png");

/// Taking `&mut World` makes this an exclusive system, pinning it to the
/// thread that runs the schedule — the main thread under `bevy_winit`'s
/// runner — which `WINIT_WINDOWS` requires, since it is a thread-local.
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
    let image = image::load_from_memory(ICON_BYTES)?.into_rgba8();
    let (width, height) = image.dimensions();
    let icon = Icon::from_rgba(image.into_raw(), width, height)?;
    Ok(icon)
}
