use dioxus::prelude::*;

use crate::{
    Route,
    ui::{alert, button::Button, card, input::Input, label::Label},
};

#[component]
pub fn Login() -> Element {
    rsx! {
      div { class: "flex h-screen items-center justify-center",
        div { class: "w-[400px]",
          div {
            alert::Root { class: "mb-4",
              alert::Title { "Registered" }

              alert::Description { "Your account has been created." }
            }

            card::Root {
              card::Header {
                card::Title { "Login" }

                card::Description { "Login to your account" }
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

                  Button { "Login" }
                }
              }

              card::Footer { class: "flex-col items-stretch gap-4",
                div { class: "self-center text-sm",
                  "Don't have an account? "
                  Link {
                    class: "underline underline-offset-4",
                    to: Route::Register {},
                    "Sign up"
                  }
                }
              }
            }
          }
        }
      }
    }
}
