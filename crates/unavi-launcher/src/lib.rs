use std::sync::LazyLock;

use config::ConfigStore;
use dioxus::native::{
    Config,
    LogicalSize,
    WindowAttributes,
    winit::{
        icon::{
            Icon,
            RgbaIcon,
        },
        window::WindowButtons,
    },
};
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

const ICON_BYTES: &[u8] = include_bytes!("../../../assets/icon-rounded.png");

fn load_icon() -> Icon {
    let image = image::load_from_memory(ICON_BYTES)
        .expect("failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    RgbaIcon::new(rgba, width, height)
        .expect("failed to create icon")
        .into()
}

pub fn run_launcher() {
    // launch_cfg bypasses dioxus::launch, which is what would otherwise
    // install this.
    dioxus::logger::initialize_default();

    let width = 380;
    let phi = std::f32::consts::GOLDEN_RATIO;

    let size = LogicalSize::new(width, (width as f32 * phi).round() as i32);

    let config = Config::new().with_window_attributes(
        WindowAttributes::default()
            .with_title("UNAVI Launcher")
            .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE)
            .with_resizable(false)
            .with_window_icon(Some(load_icon()))
            .with_surface_size(size)
            .with_min_surface_size(size)
            .with_max_surface_size(size),
    );

    dioxus::native::launch_cfg(ui::app::App, vec![], vec![Box::new(config)]);
}
