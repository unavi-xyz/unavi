use dioxus::prelude::*;
use tracing::error;

use super::app::Route;
use crate::update::{UpdateStatus, client};

#[component]
pub fn Play() -> Element {
    let mut launch_error = use_signal(|| None::<String>);
    let mut update_status = use_signal(|| None::<UpdateStatus>);
    let mut is_updating = use_signal(|| true);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::task::spawn_blocking(move || {
            client::update_client_with_callback(move |s| {
                let _ = tx.send(s);
            })
        });

        while let Some(status) = rx.recv().await {
            update_status.set(Some(status));
        }

        match handle.await {
            Ok(Ok(())) => {
                is_updating.set(false);
            }
            Ok(Err(e)) => {
                error!("Error checking client updates: {e:?}");
                update_status.set(Some(UpdateStatus::Error(format!("{e}"))));
                is_updating.set(false);
            }
            Err(e) => {
                error!("Task error: {e:?}");
                update_status.set(Some(UpdateStatus::Error(format!("task error: {e}"))));
                is_updating.set(false);
            }
        }
    });

    let handle_launch = move |_| match client::launch_client() {
        Ok(()) => {
            launch_error.set(None);
        }
        Err(e) => {
            error!("Failed to launch client: {e:?}");
            launch_error.set(Some(format!("{e}")));
        }
    };

    let nav = navigator();

    rsx! {
        div { class: "container",
            h1 { "UNAVI Launcher" }

            if let Some(ref err) = *launch_error.read() {
                div { class: "error", "failed to launch client: {err}" }
            }

            {match update_status.read().as_ref() {
                Some(UpdateStatus::Checking) => rsx! {
                    div { class: "status",
                        span { class: "loading" },
                        "checking for updates..."
                    }
                },
                Some(UpdateStatus::Downloading { version, progress }) => rsx! {
                    div { class: "status",
                        span { class: "loading" },
                        {
                            if let Some(p) = progress {
                                format!("downloading v{version} ({:.0}%)", p)
                            } else {
                                format!("downloading v{version}...")
                            }
                        }
                    }
                },
                Some(UpdateStatus::UpToDate) => rsx! {
                    div { class: "status", "up to date" }
                },
                Some(UpdateStatus::UpdatedNeedsRestart) => rsx! {
                    div { class: "status", "updated successfully" }
                },
                Some(UpdateStatus::Offline) => rsx! {
                    div { class: "status", "⚠ offline mode" }
                },
                Some(UpdateStatus::Error(e)) => rsx! {
                    div { class: "error", "{e}" }
                },
                None => rsx! {}
            }}

            div { style: "margin: 20px 0;",
                button {
                    onclick: handle_launch,
                    disabled: is_updating(),
                    "Launch Client"
                }
            }

            div { class: "version",
                "launcher v{env!(\"CARGO_PKG_VERSION\")}"
                {
                    if let Some(client_ver) = client::installed_client_version() {
                        rsx! { " • client v{client_ver}" }
                    } else {
                        rsx! {}
                    }
                }
            }

            div { style: "margin-top: 20px;",
                button {
                    onclick: move |_| {
                        nav.push(Route::Settings);
                    },
                    "Settings"
                }
            }
        }
    }
}
