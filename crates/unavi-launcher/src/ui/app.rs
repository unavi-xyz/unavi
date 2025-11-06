use dioxus::prelude::*;

use super::{client_update::ClientUpdate, home::Home, self_update::SelfUpdate, settings::Settings};

const BASE_STYLES: &str = include_str!("../../styles/base.css");
const BUTTON_STYLES: &str = include_str!("../../styles/buttons.css");
const COMPONENT_STYLES: &str = include_str!("../../styles/components.css");
const LAYOUT_STYLES: &str = include_str!("../../styles/layout.css");
const PAGE_STYLES: &str = include_str!("../../styles/pages.css");

const LOGO: Asset = asset!("/assets/logo-clear.png");

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(Title)]
    #[route("/")]
    SelfUpdate,
    #[route("/client-update")]
    ClientUpdate,
    #[route("/home")]
    Home,
    #[route("/settings")]
    Settings,
}

#[component]
pub fn Title() -> Element {
    rsx! {
        div { class: "container",
            img { class: "logo", src: LOGO }
            div { class: "content", Outlet::<Route> {} }
        }
    }
}

#[component]
pub fn App() -> Element {
    rsx! {
        document::Title { "UNAVI Launcher" }
        style { {BASE_STYLES} {LAYOUT_STYLES} {BUTTON_STYLES} {COMPONENT_STYLES} {PAGE_STYLES} }
        Router::<Route> {}
    }
}
