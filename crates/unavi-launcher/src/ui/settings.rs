use dioxus::prelude::*;
use tracing::error;

use crate::{CONFIG, config::UpdateChannel};

use super::app::Route;

#[component]
pub fn Settings() -> Element {
    let nav = navigator();
    let mut current_beta = use_signal(|| crate::CONFIG.get().update_channel.is_beta());
    let initial_beta = use_hook(|| current_beta);

    let toggle_beta = move |_| {
        if let Err(e) = CONFIG.update(|c| {
            c.update_channel = if c.update_channel.is_beta() {
                UpdateChannel::Stable
            } else {
                UpdateChannel::Beta
            }
        }) {
            error!("Failed to save config: {e}");
        } else {
            current_beta.set(!current_beta());
        }
    };

    let handle_back = move |_| {
        if current_beta == initial_beta {
            nav.push(Route::Home);
        } else {
            nav.push(Route::SelfUpdate);
        }
    };

    rsx! {
        div { class: "settings",
            label {
                input {
                    r#type: "checkbox",
                    checked: current_beta,
                    onchange: toggle_beta,
                }
                " Beta releases"
            }
        }

        button {
            class: "nav-button",
            onclick: handle_back,
            style: "margin-top: 40px;",
            "Back"
        }
    }
}
