use dioxus::prelude::*;

use super::{
    client_update::ClientUpdate, play::Play, self_update::SelfUpdate, settings::Settings,
    styles::APP_STYLES,
};

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/settings")]
    Settings,

    #[layout(Title)]
    #[route("/")]
    SelfUpdate,
    #[route("/client-update")]
    ClientUpdate,
    #[route("/play")]
    Play,
}

#[component]
pub fn Title() -> Element {
    rsx! {
            div { class: "container",
                h1 { "UNAVI" }
        Outlet::<Route> {}
    }
        }
}

#[component]
pub fn App() -> Element {
    rsx! {
        style { {APP_STYLES} }
        Router::<Route> {}
    }
}
