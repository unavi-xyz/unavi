use dioxus::prelude::*;
use tracing::error;

#[component]
pub fn Settings() -> Element {
    let config = use_signal(|| crate::CONFIG.get());

    let toggle_beta = move |_| {
        if let Err(e) = crate::CONFIG.update(|c| {
            c.update_channel = if c.update_channel.is_beta() {
                crate::config::UpdateChannel::Stable
            } else {
                crate::config::UpdateChannel::Beta
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
                " Enable beta releases"
            }
        }
    }
}
