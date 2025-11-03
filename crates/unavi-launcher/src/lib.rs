use std::sync::LazyLock;

use config::ConfigStore;
use directories::ProjectDirs;

pub mod config;
mod ui;
mod update;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-launcher").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    std::fs::create_dir_all(dirs.data_local_dir().join("clients")).expect("clients dir");
    dirs
});

pub static CONFIG: LazyLock<ConfigStore> = LazyLock::new(ConfigStore::new);

pub fn run_launcher() {
    dioxus::launch(ui::app::App);
}
