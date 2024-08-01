use leptix_primitives::primitive;
use leptos::*;
use tailwind_fuse::*;

#[component]
pub fn Button(
    #[prop(into, optional)] variant: MaybeSignal<ButtonVariant>,
    #[prop(into, optional)] size: MaybeSignal<ButtonSize>,

    #[prop(optional)] node_ref: NodeRef<html::AnyElement>,
    #[prop(into, optional)] as_child: MaybeProp<bool>,

    #[prop(into, optional)] class: MaybeSignal<String>,
    #[prop(attrs)] attributes: Vec<(&'static str, Attribute)>,
    children: ChildrenFn,
) -> impl IntoView {
    let class = create_memo(move |_| {
        let variant = variant();
        let size = size();
        let button = ButtonClass { variant, size };
        button.with_class(class())
    });

    view! {
        <primitive::Primitive
            {..attributes}
            element=html::button
            node_ref=node_ref
            as_child=as_child
            attr:class=class
        >
            {children()}
        </primitive::Primitive>
    }
}

#[derive(Clone, Copy, TwClass)]
#[tw(
    class = "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50"
)]
struct ButtonClass {
    variant: ButtonVariant,
    size: ButtonSize,
}

#[allow(dead_code)]
#[derive(TwVariant)]
pub enum ButtonVariant {
    #[tw(
        default,
        class = "bg-primary text-primary-foreground hover:bg-primary/90"
    )]
    Default,
    #[tw(class = "bg-destructive text-destructive-foreground hover:bg-destructive/90")]
    Destructive,
    #[tw(class = "border border-input bg-background hover:bg-accent hover:text-accent-foreground")]
    Outline,
    #[tw(class = "bg-secondary text-secondary-foreground hover:bg-secondary/80")]
    Secondary,
    #[tw(class = "hover:bg-accent hover:text-accent-foreground")]
    Ghost,
    #[tw(class = "text-primary underline-offset-4 hover:underline")]
    Link,
}

#[allow(dead_code)]
#[derive(TwVariant)]
pub enum ButtonSize {
    #[tw(default, class = "h-10 px-4 py-2")]
    Default,
    #[tw(class = "h-9 rounded-md px-3")]
    Sm,
    #[tw(class = "h-11 rounded-md px-8")]
    Lg,
    #[tw(class = "h-10 w-10")]
    Icon,
}
