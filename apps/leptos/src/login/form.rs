use colette_api::{
    auth::{self, LoginResponse},
    profile::Profile,
};
use leptos::prelude::*;

#[component]
pub fn LoginForm() -> impl IntoView {
    let profile = expect_context::<RwSignal<Option<Profile>>>();

    let submit = ServerAction::<Login>::new();

    Effect::new(move || {
        if let Some(Ok(LoginResponse::Ok(resp))) = submit.value().read().as_ref() {
            profile.set(Some(resp.clone()));
        }
    });

    view! {
        <ActionForm action=submit>
            <div class="card card-bordered">
                <div class="card-body">
                    <h3 class="card-title">"Login"</h3>
                    <p class="text-sm text-neutral">"Login to your account"</p>
                    <div class="form-control">
                        <div class="label">
                            <span class="label-text">"Email"</span>
                        </div>
                        <input
                            class="input input-bordered"
                            type="text"
                            name="data[email]"
                            placeholder="user@email.com"
                        />
                    </div>
                    <div class="form-control">
                        <div class="label">
                            <span class="label-text">"Password"</span>
                        </div>
                        <input
                            class="input input-bordered"
                            type="password"
                            name="data[password]"
                            placeholder="********"
                        />
                    </div>
                    <div class="card-actions">
                        <button
                            class="btn btn-primary flex-1"
                            type="submit"
                            disabled=move || submit.pending().get()
                        >
                            "Submit"
                        </button>
                    </div>
                </div>
            </div>
        </ActionForm>
    }
}

#[server]
async fn login(data: auth::Login) -> Result<LoginResponse, ServerFnError> {
    use axum::{extract::State, Json};
    use colette_api::ApiState;
    use colette_core::auth::AuthService;
    use leptos_axum::{extract_with_state, redirect};
    use tower_sessions::Session;

    let state = expect_context::<ApiState>();
    let session: Session = extract_with_state(&state).await?;
    let State(state): State<auth::AuthState> = extract_with_state(&state).await?;
    let state: State<AuthService> = extract_with_state(&state).await?;

    let resp = auth::login(state, session, Json(data)).await?;

    redirect("/");

    Ok(resp)
}
