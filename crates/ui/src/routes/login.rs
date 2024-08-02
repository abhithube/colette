use leptos::*;
use leptos_router::ActionForm;

use crate::components::{loading_spinner::LoadingSpinner, ui::{Button, Input, Label}};

#[server(Login, "/api")]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    println!("{}, {}", email, password);

    Ok(())
}

#[component]
pub fn Login() -> impl IntoView {
    let login = create_server_action::<Login>();

    view! {
        <ActionForm action=login>
            <Label for_html="email">"Email"</Label>
            <Input attr:name="email" />
            <Label for_html="password">"Password"</Label>
            <Input attr:name="password" attr:type="password" />
            <Button attr:type="submit" attr:disabled=login.pending()>
                <LoadingSpinner enabled=login.pending() />
                Login
            </Button>
        </ActionForm>
    }
}