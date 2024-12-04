use leptos::prelude::*;
use tailwind_fuse::*;

#[component]
pub fn Label(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    let class = move || {
        tw_merge!("text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70", class.get())
    };

    view! { <label class=class>{children()}</label> }
}
