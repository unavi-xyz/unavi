use dioxus::prelude::*;

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
