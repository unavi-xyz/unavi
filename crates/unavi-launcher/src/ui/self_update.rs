use dioxus::prelude::*;
use tracing::error;

use super::app::Route;

use crate::update::{UpdateStatus, launcher};

#[component]
pub fn SelfUpdate() -> Element {
    let nav = navigator();
    let mut status = use_signal(|| UpdateStatus::Checking);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::task::spawn_blocking(move || {
            launcher::update_launcher_with_callback(move |s| {
                let _ = tx.send(s);
            })
        });

        while let Some(update_status) = rx.recv().await {
            status.set(update_status);
        }

        match handle.await {
            Ok(Ok(())) => {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                nav.push(Route::ClientUpdate);
            }
            Ok(Err(e)) => {
                error!("Error updating launcher: {e:?}");
                status.set(UpdateStatus::Error(format!("launcher update failed: {e}")));
            }
            Err(e) => {
                error!("Task error: {e:?}");
                status.set(UpdateStatus::Error(format!("launcher update error: {e}")));
            }
        }
    });

    let status_text = match status() {
        UpdateStatus::Checking => "checking for updates...",
        UpdateStatus::Downloading {
            version: _,
            progress,
        } => {
            return rsx! {
                div { class: "status",
                    span { class: "loading" }
                    {
                        if let Some(p) = progress {
                            format!("downloading launcher ({:.0}%)", p)
                        } else {
                            "downloading launcher...".to_string()
                        }
                    }
                }
            };
        }
        UpdateStatus::UpToDate => "up to date, launching...",
        UpdateStatus::UpdatedNeedsRestart => "updated, please restart",
        UpdateStatus::Offline => {
            return rsx! {
                div { class: "status", "offline, skipping update" }
                button {
                    onclick: move |_| {
                        nav.push(Route::ClientUpdate);
                    },
                    "Continue"
                }
            };
        }
        UpdateStatus::Error(ref e) => {
            return rsx! {
                div { class: "error", "{e}" }
                button {
                    onclick: move |_| {
                        nav.push(Route::ClientUpdate);
                    },
                    "Continue Anyway"
                }
            };
        }
    };

    rsx! {
        div { class: "status",
            span { class: "loading" }
            "{status_text}"
        }
    }
}
