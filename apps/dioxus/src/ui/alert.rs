use dioxus::prelude::*;
use tailwind_fuse::*;

#[derive(Clone, Copy, TwClass)]
#[tw(
    class = "relative w-full rounded-lg border px-4 py-3 text-sm grid has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] grid-cols-[0_1fr] has-[>svg]:gap-x-3 gap-y-0.5 items-start [&>svg]:size-4 [&>svg]:translate-y-0.5 [&>svg]:text-current"
)]
struct AlertClass {
    variant: AlertVariant,
}

#[allow(dead_code)]
#[derive(PartialEq, TwVariant)]
pub enum AlertVariant {
    #[tw(default, class = "bg-card text-card-foreground")]
    Default,
    #[tw(
        class = "text-destructive bg-card [&>svg]:text-current *:data-[scope=alert]:*:data-[part=description]:text-destructive/90"
    )]
    Destructive,
}

#[component]
pub fn Root(
    #[props(default = AlertVariant::Default)] variant: AlertVariant,
    class: Option<String>,
    #[props(extends = div)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      div {
        class: AlertClass { variant }.with_class(class.unwrap_or_default()),
        role: "alert",
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
        class: tw_merge!("col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight", class),
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
        class: tw_merge!(
            "text-muted-foreground col-start-2 grid justify-items-start gap-1 text-sm [&_p]:leading-relaxed",
            class
        ),
        ..attributes,
        {children}
      }
    }
}
