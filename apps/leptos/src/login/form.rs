use colette_api::auth::{self, LoginResponse};
use leptos::prelude::*;

use crate::common::ui::{button::Button, card, input::Input, label::Label};

#[component]
pub fn LoginForm() -> impl IntoView {
    let submit = ServerAction::<Login>::new();

    view! {
        <ActionForm action=submit>
            <card::Root>
                <card::Header class="space-y-2">
                    <card::Title>"Login"</card::Title>
                    <card::Description>"Login to your account"</card::Description>
                </card::Header>
                <card::Content class="space-y-4">
                    <div class="space-y-2">
                        <Label>"Email"</Label>
                        <Input {..} type="text" name="data[email]" placeholder="user@email.com" />
                    </div>
                    <div class="space-y-2">
                        <Label>"Password"</Label>
                        <Input {..} type="password" name="data[password]" placeholder="********" />
                    </div>
                </card::Content>
                <card::Footer>
                    <Button
                        class="flex-1"
                        {..}
                        type="submit"
                        disabled=move || submit.pending().get()
                    >
                        "Submit"
                    </Button>
                </card::Footer>
            </card::Root>
        </ActionForm>
    }
}

#[server]
async fn login(data: auth::Login) -> Result<LoginResponse, ServerFnError> {
    use axum::{extract::State, Json};
    use colette_api::ApiState;
    use colette_core::auth::AuthService;
    use leptos_axum::extract_with_state;
    use tower_sessions::Session;

    let state = expect_context::<ApiState>();
    let session: Session = extract_with_state(&state).await?;
    let State(state): State<auth::AuthState> = extract_with_state(&state).await?;
    let state: State<AuthService> = extract_with_state(&state).await?;

    let resp = auth::login(state, session, Json(data)).await?;

    Ok(resp)
}
