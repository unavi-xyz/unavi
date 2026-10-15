use dioxus::prelude::*;
use tracing::error;

use crate::{
    CONFIG,
    update::app,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
pub fn Home() -> Element {
    let mut launch_error = use_signal(|| None::<String>);
    let mut app_running = use_signal(|| crate::APP_PROCESS.is_running());
    let mut xr_mode = use_signal(|| CONFIG.get().xr_mode);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let is_running = crate::APP_PROCESS.is_running();
            app_running.set(is_running);
        }
    });

    let mut handle_launch = move |_| match app::launch_app() {
        Ok(()) => {
            launch_error.set(None);
            app_running.set(true);

            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                std::process::exit(0);
            });
        }
        Err(e) => {
            error!("Failed to launch app: {e:?}");
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
            class: if app_running() { "home-button running" } else { "home-button" },
            onclick: move |e| {
                if !app_running() {
                    handle_launch(e);
                }
            },
            {if app_running() { "⟨  connected  ⟩" } else { "Enter" }}
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
                app::installed_app_version().map_or_else(
                    || rsx! { div { "app not found" } },
                    |app_ver| rsx! { div { "app v{app_ver}" } }
                )
            }
        }
    }
}
