use dioxus::prelude::*;
use tracing::error;

use super::app::Route;
use crate::update::{UpdateStatus, client};

#[component]
pub fn ClientUpdate() -> Element {
    let nav = navigator();
    let mut update_status = use_signal(|| None::<UpdateStatus>);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            client::update_client_with_callback(move |s| {
                let _ = tx.send(s);
            })
            .await
        });

        while let Some(status) = rx.recv().await {
            update_status.set(Some(status));
        }

        match handle.await {
            Ok(Ok(())) => {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                nav.push(Route::Home);
            }
            Ok(Err(e)) => {
                error!("Error checking client updates: {e:?}");
                update_status.set(Some(UpdateStatus::Error(format!(
                    "client update failed: {e}"
                ))));
            }
            Err(e) => {
                error!("Task error: {e:?}");
                update_status.set(Some(UpdateStatus::Error(format!(
                    "client update failed: {e}"
                ))));
            }
        }
    });

    let status_text = match update_status.read().as_ref() {
        Some(UpdateStatus::Checking) => "checking for updates...",
        Some(UpdateStatus::Downloading { version, progress }) => {
            return rsx! {
                div { class: "status",
                    span { class: "loading" }
                    {
                        if let Some(p) = progress {
                            format!("downloading v{version} ({:.0}%)", p)
                        } else {
                            format!("downloading v{version}...")
                        }
                    }
                }
            };
        }
        Some(UpdateStatus::UpToDate) => "client up to date",
        Some(UpdateStatus::UpdatedNeedsRestart) => "client updated successfully",
        Some(UpdateStatus::Offline) => {
            return rsx! {
                div { class: "status", "offline, skipping update" }
            };
        }
        Some(UpdateStatus::Error(e)) => {
            return rsx! {
                div { class: "error", "{e}" }
                button {
                    class: "nav-button",
                    onclick: move |_| {
                        nav.push(Route::Home);
                    },
                    "Continue Anyway"
                }
            };
        }
        None => "checking for updates...",
    };

    rsx! {
        div { class: "status",
            span { class: "loading" }
            "{status_text}"
        }
    }
}
