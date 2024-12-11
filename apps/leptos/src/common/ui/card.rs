use leptos::prelude::*;
use tailwind_fuse::*;

#[component]
pub fn Root(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div class=move || {
            tw_merge!(
                "rounded-lg border bg-card text-card-foreground shadow-sm",
          class.get()
            )
        }>{children()}</div>
    }
}

#[component]
pub fn Header(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <div class=move || {
            tw_merge!("flex flex-col space-y-1.5 p-6", class.get())
        }>{children()}</div>
    }
}

#[component]
pub fn Title(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    view! {
        <h3 class=move || {
            tw_merge!("text-2xl font-semibold leading-none tracking-tight",
          class.get())
        }>{children()}</h3>
    }
}

#[component]
pub fn Description(
    #[prop(into, optional)] class: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! { <p class=move || tw_merge!("text-sm text-muted-foreground", class.get())>{children()}</p> }
}

#[component]
pub fn Content(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    view! { <div class=move || tw_merge!("p-6 pt-0", class.get())>{children()}</div> }
}

#[component]
pub fn Footer(#[prop(into, optional)] class: Signal<String>, children: Children) -> impl IntoView {
    view! { <div class=move || tw_merge!("flex items-center p-6 pt-0", class.get())>{children()}</div> }
}
