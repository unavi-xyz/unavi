use dioxus::prelude::*;
use tracing::error;

use crate::CONFIG;

#[component]
pub fn XrToggle() -> Element {
    let mut xr_mode = use_signal(|| CONFIG.get().xr_mode);

    let toggle = move |_| {
        if let Err(e) = CONFIG.update(|c| {
            c.xr_mode = !c.xr_mode;
        }) {
            error!("Failed to save config: {e}");
        } else {
            xr_mode.set(!xr_mode());
        }
    };

    rsx! {
        div { class: "settings",
            button { class: "xr-toggle", onclick: toggle,
                div { class: "cells",
                    span { class: if xr_mode() { "cell" } else { "cell on" } }
                    span { class: if xr_mode() { "cell on" } else { "cell" } }
                }
                "XR mode"
            }
        }
    }
}
