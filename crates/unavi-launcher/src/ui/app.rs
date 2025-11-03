use dioxus::prelude::*;

use super::{
    client_update::ClientUpdate, play::Play, self_update::SelfUpdate, styles::APP_STYLES,
};

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    SelfUpdate,
    #[route("/client-update")]
    ClientUpdate,
    #[route("/play")]
    Play,
}

#[component]
pub fn App() -> Element {
    rsx! {
        style { {APP_STYLES} }
        Router::<Route> {}
    }
}
