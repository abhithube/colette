use leptos::*;

use crate::components::ui::icons::LoaderCircle;

#[component]
pub fn LoadingSpinner(#[prop(into, optional)] enabled: MaybeSignal<bool>) -> impl IntoView {
    view! {
        <Show when=enabled>
            <LoaderCircle attr:class="mr-2 h-4 w-4 animate-spin" />
        </Show>
    }
}
