use std::sync::LazyLock;

use config::ConfigStore;
use dioxus::desktop::{LogicalSize, WindowBuilder, tao::window::Icon};
use directories::ProjectDirs;
use process::ProcessTracker;

pub mod config;
pub mod process;
mod ui;
mod update;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-launcher").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    std::fs::create_dir_all(dirs.data_local_dir().join("clients")).expect("clients dir");
    dirs
});

pub static CONFIG: LazyLock<ConfigStore> = LazyLock::new(ConfigStore::new);
pub static CLIENT_PROCESS: LazyLock<ProcessTracker> = LazyLock::new(ProcessTracker::new);

fn load_icon() -> Icon {
    let icon_bytes = include_bytes!("../assets/unavi-rounded.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("failed to create icon")
}

pub fn run_launcher() {
    let bg = (0, 0, 0, 255);
    let icon = load_icon();

    let width = 380;
    let phi = 1.61803;

    #[allow(clippy::cast_possible_truncation)]
    let size = LogicalSize::new(width, (width as f32 * phi).round() as i32);

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_menu(None)
                .with_background_color(bg)
                .with_data_directory(DIRS.data_local_dir())
                .with_icon(icon.clone())
                .with_window(
                    WindowBuilder::new()
                        .with_title("UNAVI Launcher")
                        .with_maximizable(false)
                        .with_resizable(false)
                        .with_background_color(bg)
                        .with_window_icon(Some(icon))
                        .with_inner_size(size)
                        .with_min_inner_size(size),
                ),
        )
        .launch(ui::app::App);
}
