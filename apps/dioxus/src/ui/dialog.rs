use dioxus::prelude::*;
use dioxus_primitives::dialog;
use tailwind_fuse::*;

pub type DialogRootProps = dialog::DialogRootProps;

#[component]
pub fn DialogRoot(props: DialogRootProps) -> Element {
    dialog::DialogRoot(props)
}

#[component]
pub fn DialogClose(
    id: ReadOnlySignal<Option<String>>,
    class: Option<String>,
    open: Signal<bool>,
    #[props(extends = button)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      button {
        class,
        aria_label: "Close",
        tabindex: if open() { "0" } else { "-1" },
        onclick: move |_| open.set(false),
        ..attributes,
        {children}
      }
    }
}

#[component]
pub fn DialogContent(
    id: ReadOnlySignal<Option<String>>,
    class: Option<String>,
    open: Signal<bool>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      dialog::DialogContent {
        id,
        class: tw_merge!(
            "bg-background data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border p-6 shadow-lg duration-200 sm:max-w-lg",
            class
        ),
        attributes,
        children: rsx! {
          DialogClose {
            class: "ring-offset-background focus:ring-ring data-[state=open]:bg-accent data-[state=open]:text-muted-foreground absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
            open,
            children: rsx! {
              "×"
              
              span { class: "sr-only", "Close" }
            },
          }
          
          {children}
        },
      }
    }
}

#[component]
pub fn DialogTitle(
    id: ReadOnlySignal<Option<String>>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      dialog::DialogTitle {
        id,
        class: tw_merge!("text-lg leading-none font-semibold", class),
        attributes,
        children,
      }
    }
}

#[component]
pub fn DialogDescription(
    id: ReadOnlySignal<Option<String>>,
    class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    rsx! {
      dialog::DialogDescription {
        id,
        class: tw_merge!("text-muted-foreground text-sm", class),
        attributes,
        children,
      }
    }
}
