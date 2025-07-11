use dioxus::prelude::*;
use tailwind_fuse::*;

#[component]
pub fn Skeleton(
    class: Option<String>,
    #[props(extends = div)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      div {
        class: tw_merge!("bg-accent animate-pulse rounded-md", class),
        ..attributes,
        {children}
      }
    }
}
