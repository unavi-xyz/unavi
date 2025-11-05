use dioxus::prelude::*;
use tracing::error;

use crate::{CONFIG, config::UpdateChannel};

use super::app::Route;

#[component]
pub fn Settings() -> Element {
    let nav = navigator();
    let config = use_signal(|| crate::CONFIG.get());

    let toggle_beta = move |_| {
        if let Err(e) = CONFIG.update(|c| {
            c.update_channel = if c.update_channel.is_beta() {
                UpdateChannel::Stable
            } else {
                UpdateChannel::Beta
            }
        }) {
            error!("Failed to save config: {e}");
        }
    };

    rsx! {
        div { class: "settings",
            label {
                input {
                    r#type: "checkbox",
                    checked: config().update_channel.is_beta(),
                    onchange: toggle_beta,
                }
                " Beta releases"
            }
        }

        button {
            class: "nav-button",
            onclick: move |_| {
                nav.push(Route::Home);
            },
            style: "margin-top: 40px;",
            "Back"
        }
    }
}
