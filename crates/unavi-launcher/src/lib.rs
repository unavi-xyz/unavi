use std::sync::LazyLock;

use directories::ProjectDirs;
use dioxus::prelude::*;
use tracing::error;

mod update;

use update::launcher::UpdateStatus;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-launcher").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    std::fs::create_dir_all(dirs.data_local_dir().join("clients")).expect("clients dir");
    dirs
});

pub fn run_launcher() {
    dioxus::launch(App);
}

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    SelfUpdate,
    #[route("/play")]
    Play,
}

#[component]
fn App() -> Element {
    rsx! {
        style {
            r#"
                * {{
                    box-sizing: border-box;
                }}
                body {{
                    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Oxygen", "Ubuntu", "Cantarell", sans-serif;
                    background: #0d0d0d;
                    margin: 0;
                    padding: 0;
                    color: #f5f5f5;
                }}
                .container {{
                    background: #0d0d0d;
                    border: 1px solid #2a2a2a;
                    border-radius: 8px;
                    padding: 32px;
                    margin: 32px;
                }}
                h1 {{
                    margin: 0 0 24px 0;
                    color: #f5f5f5;
                    font-size: 18px;
                    font-weight: 500;
                    letter-spacing: -0.01em;
                }}
                .status {{
                    margin: 16px 0;
                    padding: 0;
                    color: #a0a0a0;
                    font-size: 14px;
                    line-height: 1.6;
                }}
                .loading {{
                    display: inline-block;
                    width: 14px;
                    height: 14px;
                    border: 2px solid #2a2a2a;
                    border-top: 2px solid #f5f5f5;
                    border-radius: 50%;
                    animation: spin 1s cubic-bezier(0.4, 0, 0.2, 1) infinite;
                    margin-right: 10px;
                    vertical-align: middle;
                }}
                @keyframes spin {{
                    0% {{ transform: rotate(0deg); }}
                    100% {{ transform: rotate(360deg); }}
                }}
                button {{
                    background: #f5f5f5;
                    color: #0d0d0d;
                    border: 1px solid #f5f5f5;
                    border-radius: 6px;
                    padding: 12px 20px;
                    font-size: 14px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
                    font-family: inherit;
                    width: 100%;
                }}
                button:hover {{
                    background: #0d0d0d;
                    color: #f5f5f5;
                    transform: translateY(-1px);
                }}
                button:active {{
                    transform: translateY(0);
                }}
                button:disabled {{
                    background: #2a2a2a;
                    border-color: #2a2a2a;
                    color: #666;
                    cursor: not-allowed;
                    transform: none;
                }}
                .error {{
                    background: #1a1a1a;
                    border: 1px solid #f5f5f5;
                    border-radius: 6px;
                    color: #f5f5f5;
                    padding: 14px;
                    margin: 16px 0;
                    font-size: 13px;
                    line-height: 1.5;
                }}
                .version {{
                    color: #666;
                    font-size: 12px;
                    margin-top: 24px;
                }}
            "#
        }

        Router::<Route> {}
    }
}

#[component]
fn SelfUpdate() -> Element {
    let nav = navigator();
    let mut status = use_signal(|| UpdateStatus::Checking);

    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::task::spawn_blocking(move || {
            update::launcher::update_launcher_with_callback(move |s| {
                let _ = tx.send(s);
            })
        });

        // Process status updates
        while let Some(update_status) = rx.recv().await {
            status.set(update_status);
        }

        // Check final result
        match handle.await {
            Ok(Ok(())) => {
                if *status.read() == UpdateStatus::UpToDate {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    nav.push(Route::Play);
                }
            }
            Ok(Err(e)) => {
                error!("Error updating launcher: {e:?}");
                status.set(UpdateStatus::Error(format!("{e}")));
            }
            Err(e) => {
                error!("Task error: {e:?}");
                status.set(UpdateStatus::Error(format!("task error: {e}")));
            }
        }
    });

    let status_text = match status() {
        UpdateStatus::Checking => "checking for updates...",
        UpdateStatus::Downloading(ref version) => {
            return rsx! {
                div { class: "container",
                    h1 { "UNAVI Launcher" }
                    div { class: "status",
                        span { class: "loading" }
                        "downloading version {version}..."
                    }
                }
            };
        }
        UpdateStatus::UpToDate => "up to date, launching...",
        UpdateStatus::UpdatedNeedsRestart => "updated, please restart",
        UpdateStatus::Error(ref e) => {
            return rsx! {
                div { class: "container",
                    h1 { "UNAVI Launcher" }
                    div { class: "error", "error: {e}" }
                    button {
                        onclick: move |_| {
                            nav.push(Route::Play);
                        },
                        "Continue Anyway"
                    }
                }
            };
        }
    };

    rsx! {
        div { class: "container",
            h1 { "UNAVI Launcher" }
            div { class: "status",
                span { class: "loading" }
                "{status_text}"
            }
        }
    }
}

#[component]
fn Play() -> Element {
    let mut launch_error = use_signal(|| None::<String>);
    let mut update_status = use_signal(|| None::<UpdateStatus>);
    let mut is_updating = use_signal(|| true);

    // Auto-update client on mount
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        // Small delay to let UI settle
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::task::spawn_blocking(move || {
            update::client::update_client_with_callback(move |s| {
                let _ = tx.send(s);
            })
        });

        // Process status updates
        while let Some(status) = rx.recv().await {
            update_status.set(Some(status));
        }

        // Check final result
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

    let handle_launch = move |_| match update::client::launch_client() {
        Ok(()) => {
            launch_error.set(None);
        }
        Err(e) => {
            error!("Failed to launch client: {e:?}");
            launch_error.set(Some(format!("{e}")));
        }
    };


    rsx! {
        div { class: "container",
            h1 { "UNAVI Launcher" }

            if let Some(ref err) = *launch_error.read() {
                div { class: "error", "failed to launch client: {err}" }
            }

            // Show update status
            {match update_status.read().as_ref() {
                Some(UpdateStatus::Checking) => rsx! {
                    div { class: "status",
                        span { class: "loading" },
                        "checking for updates..."
                    }
                },
                Some(UpdateStatus::Downloading(version)) => rsx! {
                    div { class: "status",
                        span { class: "loading" },
                        "downloading v{version}..."
                    }
                },
                Some(UpdateStatus::UpToDate) => rsx! {
                    div { class: "status", "up to date" }
                },
                Some(UpdateStatus::UpdatedNeedsRestart) => rsx! {
                    div { class: "status", "updated successfully" }
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
                    if let Some(client_ver) = update::client::installed_client_version() {
                        rsx! { " • client v{client_ver}" }
                    } else {
                        rsx! {}
                    }
                }
            }
        }
    }
}
