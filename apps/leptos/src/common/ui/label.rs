use leptos::prelude::*;
use tailwind_fuse::*;

#[component]
pub fn Label(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    let class = move || tw_merge!("apps/leptos/src/common/ui/input.rs", class.get());

    view! { <label class=class>{children()}</label> }
}
