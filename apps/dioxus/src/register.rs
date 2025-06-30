use dioxus::prelude::*;

use crate::{
    Route,
    ui::{button::Button, card, input::Input, label::Label},
};

#[component]
pub fn Register() -> Element {
    rsx! {
      div { class: "flex h-screen items-center justify-center",
        div { class: "w-[400px]",
          div {
            card::Root {
              card::Header {
                card::Title { "Register" }

                card::Description { "Register a new account" }
              }

              card::Content {
                form { class: "flex flex-col items-stretch space-y-4",
                  div { class: "flex flex-col gap-2",
                    Label { html_for: "email", "Email" }

                    Input {
                      id: "email",
                      r#type: "email",
                      placeholder: "user@example.com",
                    }
                  }

                  div { class: "flex flex-col gap-2",
                    Label { html_for: "password", "Password" }

                    Input {
                      id: "password",
                      r#type: "password",
                      placeholder: "********",
                    }
                  }

                  div { class: "flex flex-col gap-2",
                    Label { html_for: "confirm-password", "Confirm Password" }

                    Input {
                      id: "confirm-password",
                      r#type: "password",
                      placeholder: "********",
                    }
                  }

                  Button { "Register" }
                }
              }

              card::Footer { class: "flex-col items-stretch gap-4",
                div { class: "self-center text-sm",
                  "Already have an account? "
                  Link {
                    class: "underline underline-offset-4",
                    to: Route::Login {},
                    "Sign in"
                  }
                }
              }
            }
          }
        }
      }
    }
}
