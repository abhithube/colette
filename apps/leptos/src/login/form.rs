use colette_api::auth;
use leptos::prelude::*;

#[component]
pub fn LoginForm() -> impl IntoView {
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
async fn login(data: auth::Login) -> Result<auth::LoginResponse, ServerFnError> {
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
