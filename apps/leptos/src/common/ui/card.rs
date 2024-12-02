use leptos::prelude::*;
use tailwind_fuse::*;

#[component]
pub fn Root(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    let class = move || {
        tw_merge!(
            "rounded-lg border bg-card text-card-foreground shadow-sm",
            class.get()
        )
    };

    view! { <div class=class>{children()}</div> }
}

#[component]
pub fn Header(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    let class = move || tw_merge!("flex flex-col space-y-1.5 p-6", class.get());

    view! { <div class=class>{children()}</div> }
}

#[component]
pub fn Title(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    let class = move || {
        tw_merge!(
            "text-2xl font-semibold leading-none tracking-tight",
            class.get()
        )
    };

    view! { <h3 class=class>{children()}</h3> }
}

#[component]
pub fn Description(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let class = move || tw_merge!("text-sm text-muted-foreground", class.get());

    view! { <p class=class>{children()}</p> }
}

#[component]
pub fn Content(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    let class = move || tw_merge!("p-6 pt-0", class.get());

    view! { <div class=class>{children()}</div> }
}

#[component]
pub fn Footer(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    let class = move || tw_merge!("flex items-center p-6 pt-0", class.get());

    view! { <div class=class>{children()}</div> }
}
