use dioxus::prelude::*;

mod update;

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
        // document::Link { rel: "icon", href: FAVICON }
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        // document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        p {
            "Root"
        }

        Router::<Route> {}
    }
}

#[component]
fn SelfUpdate() -> Element {
    use_hook(|| {
        std::thread::spawn(|| {
            if let Err(e) = update::launcher::update_launcher() {
                error!("Error updating launcher: {e:?}");
            }
        });
    });

    rsx! {
        p {
            "Self updating..."
        }
    }
}

#[component]
fn Play() -> Element {
    rsx! {
        p {
            "Play"
        }
    }
}
