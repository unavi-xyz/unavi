use dioxus::prelude::*;

use super::{
    client_update::ClientUpdate, play::Play, self_update::SelfUpdate, settings::Settings,
};

const BASE_STYLES: &str = include_str!("../../styles/base.css");
const LAYOUT_STYLES: &str = include_str!("../../styles/layout.css");
const BUTTON_STYLES: &str = include_str!("../../styles/buttons.css");
const COMPONENT_STYLES: &str = include_str!("../../styles/components.css");
const PAGE_STYLES: &str = include_str!("../../styles/pages.css");

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(Title)]
    #[route("/")]
    SelfUpdate,
    #[route("/client-update")]
    ClientUpdate,
    #[route("/play")]
    Play,
    #[route("/settings")]
    Settings,
}

#[component]
pub fn Title() -> Element {
    rsx! {
        div { class: "container",
            h1 { "UNAVI" }
            div { class: "content", Outlet::<Route> {} }
        }
    }
}

#[component]
pub fn App() -> Element {
    rsx! {
        style { {BASE_STYLES} {LAYOUT_STYLES} {BUTTON_STYLES} {COMPONENT_STYLES} {PAGE_STYLES} }
        Router::<Route> {}
    }
}
