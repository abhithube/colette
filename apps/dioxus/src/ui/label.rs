use dioxus::prelude::*;
use dioxus_primitives::label;
use tailwind_fuse::*;

#[component]
pub fn Label(
    html_for: ReadOnlySignal<String>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      label::Label {
        class: tw_merge!(
            "flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
            class
        ),
        html_for,
        {children}
      }
    }
}
