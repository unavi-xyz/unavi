use dioxus::prelude::*;
use tracing::error;

use super::app::Route;
use crate::update::client;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
pub fn Home() -> Element {
    let mut launch_error = use_signal(|| None::<String>);
    let mut client_running = use_signal(|| crate::CLIENT_PROCESS.is_running());

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

            // Close launcher after a brief delay.
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                std::process::exit(0);
            });
        }
        Err(e) => {
            error!("Failed to launch client: {e:?}");
            launch_error.set(Some(format!("{e}")));
        }
    };

    let nav = navigator();

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

        button {
            class: "nav-button",
            onclick: move |_| {
                nav.push(Route::Settings);
            },
            "Settings"
        }

        div { style: "min-height: 40px;",
            if let Some(ref err) = *launch_error.read() {
                div { class: "error", "{err}" }
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
