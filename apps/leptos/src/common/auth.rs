use colette_api::{profile, Session};
use leptos::prelude::*;
use leptos::server;

#[server]
pub async fn validate_session() -> Result<Session, ServerFnError> {
    use axum::extract::OriginalUri;
    use colette_api::ApiState;
    use leptos_axum::{extract, extract_with_state, redirect};

    let uri = extract::<OriginalUri>().await?;

    let state = expect_context::<ApiState>();
    let session: Session = match extract_with_state(&state).await {
        Ok(session) => Ok(session),
        Err(e) => {
            if uri.path() != "/login" {
                redirect("/login");
            }

            Err(e)
        }
    }?;

    if uri.path() == "/login" {
        redirect("/");

        return Err(ServerFnError::new(""));
    }

    Ok(session)
}

#[server]
pub async fn get_active_profile() -> Result<profile::GetActiveResponse, ServerFnError> {
    use axum::extract::State;
    use colette_api::{profile::ProfileState, ApiState};
    use colette_core::profile::ProfileService;
    use leptos_axum::extract_with_state;

    let state = expect_context::<ApiState>();

    let session: Session = validate_session().await?;

    let State(state): State<ProfileState> = extract_with_state(&state).await?;
    let state: State<ProfileService> = extract_with_state(&state).await?;

    let resp = profile::get_active_profile(state, session).await?;

    Ok(resp)
}
