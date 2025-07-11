use dioxus::prelude::*;
use dioxus_primitives::separator;
use tailwind_fuse::*;

#[component]
pub fn Separator(
    class: Option<String>,
    #[props(default = true)] horizontal: bool,
    #[props(default = false)] decorative: bool,
    #[props(extends = input, extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    rsx! {
      separator::Separator {
        class: tw_merge!(
            "bg-border shrink-0 data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px",
            class
        ),
        horizontal,
        decorative,
        attributes,
      }
    }
}
