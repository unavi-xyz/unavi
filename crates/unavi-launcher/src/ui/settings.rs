use dioxus::prelude::*;
use tracing::error;

use crate::config::{Config, UpdateChannel};

use super::app::Route;

#[component]
pub fn Settings() -> Element {
    let nav = navigator();
    let config = use_signal(|| crate::CONFIG.get());

    let toggle_channel = move |_| {
        let new_channel = match config().update_channel {
            UpdateChannel::Stable => UpdateChannel::Beta,
            UpdateChannel::Beta => UpdateChannel::Stable,
        };

        if let Err(e) = crate::CONFIG.update(|c| c.update_channel = new_channel) {
            error!("Failed to save config: {e}");
        } else {
            config.set(crate::CONFIG.get());
        }
    };

    let toggle_auto_close = move |_| {
        let new_value = !config().auto_close_on_launch;

        if let Err(e) = crate::CONFIG.update(|c| c.auto_close_on_launch = new_value) {
            error!("Failed to save config: {e}");
        } else {
            config.set(crate::CONFIG.get());
        }
    };

    rsx! {
        div { class: "container",
            h1 { "Settings" }

            div { class: "setting-group",
                h3 { "Update Channel" }
                button {
                    onclick: toggle_channel,
                    {
                        match config().update_channel {
                            UpdateChannel::Stable => "Stable",
                            UpdateChannel::Beta => "Beta (Preview)",
                        }
                    }
                }
                p { class: "setting-description",
                    {
                        match config().update_channel {
                            UpdateChannel::Stable => "Using stable releases",
                            UpdateChannel::Beta => "Using beta/preview releases",
                        }
                    }
                }
            }

            div { class: "setting-group",
                h3 { "Auto-close Launcher" }
                label {
                    input {
                        r#type: "checkbox",
                        checked: config().auto_close_on_launch,
                        onchange: toggle_auto_close,
                    }
                    " Close launcher when client starts"
                }
                p { class: "setting-description",
                    "Automatically close this launcher window after launching the game client"
                }
            }

            div { style: "margin-top: 30px;",
                button {
                    onclick: move |_| {
                        nav.push(Route::Play);
                    },
                    "Back to Launcher"
                }
            }
        }
    }
}
