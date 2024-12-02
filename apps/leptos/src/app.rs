use colette_api::auth;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/colette.css" />

        <Title text="Welcome to Leptos" />

        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center">
            <div class="w-[400px]">
                <LoginForm />
            </div>
        </div>
    }
}

#[component]
fn LoginForm() -> impl IntoView {
    let submit = ServerAction::<Login>::new();

    view! {
        <ActionForm action=submit>
            <div class="flex flex-col">
                <input type="text" name="data[email]" />
                <input type="password" name="data[password]" />
                <input type="submit" value="Submit" />
            </div>
        </ActionForm>
    }
}

#[server]
pub async fn login(data: auth::Login) -> Result<auth::LoginResponse, ServerFnError> {
    use crate::AppState;
    use axum::{extract::State, Json};
    use colette_core::auth::AuthService;
    use leptos_axum::extract_with_state;

    let state = expect_context::<AppState>();
    let session: tower_sessions::Session = extract_with_state(&state.api_state).await?;
    let State(state): State<auth::AuthState> = extract_with_state(&state.api_state).await?;
    let state: State<AuthService> = extract_with_state(&state).await?;

    let resp = auth::login(state, session, Json(data)).await?;

    Ok(resp)
}
