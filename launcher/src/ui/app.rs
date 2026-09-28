use dioxus::prelude::*;

use super::{
    client_update::ClientUpdate,
    home::Home,
    self_update::SelfUpdate,
    settings::Settings,
};

const BASE_STYLES: &str = include_str!("../../assets/base.css");
const BUTTON_STYLES: &str = include_str!("../../assets/buttons.css");
const COMPONENT_STYLES: &str = include_str!("../../assets/components.css");
const LAYOUT_STYLES: &str = include_str!("../../assets/layout.css");
const PAGE_STYLES: &str = include_str!("../../assets/pages.css");

const LOGO: &str = include_str!(concat!(env!("OUT_DIR"), "/logo.uri"));

#[derive(Debug, Clone, Routable, PartialEq, Eq)]
pub enum Route {
    #[layout(Title)]
    #[route("/")]
    SelfUpdate,
    #[route("/client-update")]
    ClientUpdate,
    #[route("/home")]
    Home,
    // Kept for future use; no button in the UI currently links here.
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
        document::Style { {BASE_STYLES} }
        document::Style { {LAYOUT_STYLES} }
        document::Style { {BUTTON_STYLES} }
        document::Style { {COMPONENT_STYLES} }
        document::Style { {PAGE_STYLES} }
        Router::<Route> {}
    }
}
