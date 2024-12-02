mod form;

use form::LoginForm;
use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center">
            <div class="w-[400px]">
                <LoginForm />
            </div>
        </div>
    }
}
