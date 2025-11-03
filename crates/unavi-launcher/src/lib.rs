use std::sync::LazyLock;

use directories::ProjectDirs;

mod ui;
mod update;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-launcher").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    std::fs::create_dir_all(dirs.data_local_dir().join("clients")).expect("clients dir");
    dirs
});

pub fn run_launcher() {
    dioxus::launch(ui::app::App);
}
