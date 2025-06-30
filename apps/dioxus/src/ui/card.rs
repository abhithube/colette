use dioxus::prelude::*;
use tailwind_fuse::*;

#[component]
pub fn Root(
    class: Option<String>,
    #[props(extends = div)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      div {
        class: tw_merge!(
            "bg-card text-card-foreground flex flex-col gap-6 rounded-xl border py-6 shadow-sm",
            class
        ),
        ..attributes,
        {children}
      }
    }
}

#[component]
pub fn Header(
    class: Option<String>,
    #[props(extends = div)] attributes: Vec<Attribute>,

    children: Element,
) -> Element {
    rsx! {
      div {
        class: tw_merge!(
            "@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-1.5 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6",
            class
        ),
        ..attributes,
        {children}
      }
    }
}

#[component]
pub fn Title(
    class: Option<String>,
    #[props(extends = h3)] attributes: Vec<Attribute>,

    children: Element,
) -> Element {
    rsx! {
      h3 {
        class: tw_merge!("leading-none font-semibold", class),
        ..attributes,
        {children}
      }
    }
}

#[component]
pub fn Description(
    class: Option<String>,
    #[props(extends = p)] attributes: Vec<Attribute>,

    children: Element,
) -> Element {
    rsx! {
      p {
        class: tw_merge!("text-muted-foreground text-sm", class),
        ..attributes,
        {children}
      }
    }
}

#[component]
pub fn Action(
    class: Option<String>,
    #[props(extends = div)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      div {
        class: tw_merge!("col-start-2 row-span-2 row-start-1 self-start justify-self-end", class),
        ..attributes,
        {children}
      }
    }
}

#[component]
pub fn Content(
    class: Option<String>,
    #[props(extends = div)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      div { class: tw_merge!("px-6", class), ..attributes, {children} }
    }
}

#[component]
pub fn Footer(
    class: Option<String>,
    #[props(extends = div)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      div {
        class: tw_merge!("flex items-center px-6 [.border-t]:pt-6", class),
        ..attributes,
        {children}
      }
    }
}
