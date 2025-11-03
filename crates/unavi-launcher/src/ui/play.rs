use dioxus::prelude::*;
use tracing::error;

use super::settings::Settings;
use crate::update::client;

#[component]
pub fn Play() -> Element {
    let mut launch_error = use_signal(|| None::<String>);
    let mut client_running = use_signal(|| false);

    // Poll client process status
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            let is_running = crate::CLIENT_PROCESS.is_running();
            client_running.set(is_running);
        }
    });

    let handle_launch = move |_| match client::launch_client() {
        Ok(()) => {
            launch_error.set(None);
            client_running.set(true);
        }
        Err(e) => {
            error!("Failed to launch client: {e:?}");
            launch_error.set(Some(format!("{e}")));
        }
    };

    rsx! {
        div { class: "container",
            h1 { "UNAVI" }

            button {
                class: "play-button",
                onclick: handle_launch,
                disabled: client_running(),
                {
                    if client_running() {
                        "Running"
                    } else {
                        "Play"
                    }
                }
            }

            if let Some(ref err) = *launch_error.read() {
                div { class: "error", "{err}" }
            }

            Settings {}

            div { class: "version",
                "v{env!(\"CARGO_PKG_VERSION\")}"
                {
                    if let Some(client_ver) = client::installed_client_version() {
                        rsx! { " • client v{client_ver}" }
                    } else {
                        rsx! {}
                    }
                }
            }
        }
    }
}
