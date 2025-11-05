use std::sync::LazyLock;

use config::ConfigStore;
use dioxus::desktop::{LogicalSize, WindowBuilder};
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

pub fn run_launcher() {
    let bg = (0, 0, 0, 255);

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_menu(None)
                .with_background_color(bg)
                .with_data_directory(DIRS.data_local_dir())
                .with_window(
                    WindowBuilder::new()
                        .with_title("UNAVI Launcher")
                        .with_maximizable(false)
                        .with_resizable(false)
                        .with_background_color(bg)
                        .with_inner_size(LogicalSize::new(400, 500))
                        .with_min_inner_size(LogicalSize::new(400, 500)),
                ),
        )
        .launch(ui::app::App);
}
