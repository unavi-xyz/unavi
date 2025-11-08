use dioxus::prelude::*;

use super::{client_update::ClientUpdate, home::Home, self_update::SelfUpdate, settings::Settings};

const BASE_STYLES: Asset = asset!("/assets/base.css");
const BUTTON_STYLES: Asset = asset!("/assets/buttons.css");
const COMPONENT_STYLES: Asset = asset!("/assets/components.css");
const LAYOUT_STYLES: Asset = asset!("/assets/layout.css");
const PAGE_STYLES: Asset = asset!("/assets/pages.css");

const LOGO: Asset = asset!("/assets/unavi-clear.png");

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
        document::Link { rel: "stylesheet", href: BASE_STYLES }
        document::Link { rel: "stylesheet", href: LAYOUT_STYLES }
        document::Link { rel: "stylesheet", href: BUTTON_STYLES }
        document::Link { rel: "stylesheet", href: COMPONENT_STYLES }
        document::Link { rel: "stylesheet", href: PAGE_STYLES }
        Router::<Route> {}
    }
}
