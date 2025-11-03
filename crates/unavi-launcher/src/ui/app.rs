use dioxus::prelude::*;

use super::{play::Play, self_update::SelfUpdate, settings::Settings, styles::APP_STYLES};

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    SelfUpdate,
    #[route("/play")]
    Play,
    #[route("/settings")]
    Settings,
}

#[component]
pub fn App() -> Element {
    rsx! {
        style { {APP_STYLES} }
        Router::<Route> {}
    }
}
