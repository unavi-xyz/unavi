use dioxus::prelude::*;

use super::{
    app::Route,
    xr_toggle::XrToggle,
};

#[component]
pub fn Settings() -> Element {
    let nav = navigator();
    rsx! {
        XrToggle {}

        button {
            class: "nav-button spaced",
            onclick: move |_| { nav.push(Route::Home); },
            "Back"
        }
    }
}
