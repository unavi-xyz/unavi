use dioxus::prelude::*;
use tracing::error;

use super::app::Route;
use crate::CONFIG;

#[component]
pub fn Settings() -> Element {
    let nav = navigator();
    let mut xr_mode = use_signal(|| CONFIG.get().xr_mode);

    let toggle_xr = move |_| {
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
            label {
                input {
                    r#type: "checkbox",
                    checked: xr_mode,
                    onchange: toggle_xr,
                }
                " XR mode"
            }
        }

        button {
            class: "nav-button",
            onclick: move |_| { nav.push(Route::Home); },
            style: "margin-top: 40px;",
            "Back"
        }
    }
}
