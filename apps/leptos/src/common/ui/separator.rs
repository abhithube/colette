use leptos::prelude::*;
use tailwind_fuse::*;

#[component]
pub fn Separator(
    #[prop(into, optional)] orientation: Signal<Orientation>,

    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let class = move || {
        tw_merge!(
            "shrink-0 bg-border",
            if orientation.get() == Orientation::Horizontal {
                "h-[1px] w-full"
            } else {
                "h-full w-[1px]"
            },
            class.get()
        )
    };

    view! { <div class=class /> }
}

#[allow(dead_code)]
#[derive(Clone, Default, PartialEq)]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}
