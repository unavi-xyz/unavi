use dioxus::prelude::*;
use tracing::error;

use crate::{
    CONFIG,
    update::client,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
pub fn Home() -> Element {
    let mut launch_error = use_signal(|| None::<String>);
    let mut client_running = use_signal(|| crate::CLIENT_PROCESS.is_running());
    let mut xr_mode = use_signal(|| CONFIG.get().xr_mode);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let is_running = crate::CLIENT_PROCESS.is_running();
            client_running.set(is_running);
        }
    });

    let mut handle_launch = move |_| match client::launch_client() {
        Ok(()) => {
            launch_error.set(None);
            client_running.set(true);

            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                std::process::exit(0);
            });
        }
        Err(e) => {
            error!("Failed to launch client: {e:?}");
            launch_error.set(Some(e.to_string()));
        }
    };

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
        button {
            class: if client_running() { "home-button running" } else { "home-button" },
            onclick: move |e| {
                if !client_running() {
                    handle_launch(e);
                }
            },
            {if client_running() { "⟨  connected  ⟩" } else { "Enter" }}
        }

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

        // Unreachable from the UI; no button currently links here.
        /*
        let nav = navigator();
        button {
            class: "nav-button",
            onclick: move |_| {
                nav.push(Route::Settings);
            },
            "Settings"
        }
        */

        div { style: "min-height: 40px;",
            if let Some(ref err) = *launch_error.read() {
                div { class: "error", {err.as_str()} }
            }
        }

        div { class: "version",
            div { "launcher v{VERSION}" }
            {
                client::installed_client_version().map_or_else(
                    || rsx! { div { "client not found" } },
                    |client_ver| rsx! { div { "client v{client_ver}" } }
                )
            }
        }
    }
}
