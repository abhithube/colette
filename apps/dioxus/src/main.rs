use dioxus::prelude::*;

use crate::{login::Login, register::Register};

mod login;
mod register;
mod ui;

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div { class: "flex gap-1",
            Link { to: Route::Login {}, "Login" }
            Link { to: Route::Register {}, "Register" }
        }

        h1 { "Home" }
    }
}
