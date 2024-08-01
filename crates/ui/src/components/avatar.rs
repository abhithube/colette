use leptix_primitives::avatar;
use leptos::*;
use tailwind_fuse::*;

#[component]
pub fn Avatar(
    #[prop(optional)] node_ref: NodeRef<html::AnyElement>,
    #[prop(optional, into)] as_child: MaybeProp<bool>,

    #[prop(into, optional)] class: MaybeSignal<String>,
    #[prop(attrs)] attributes: Vec<(&'static str, Attribute)>,
    children: ChildrenFn,
) -> impl IntoView {
    let class = create_memo(move |_| {
        tw_merge!(
            "relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full",
            class()
        )
    });

    view! {
        <avatar::AvatarRoot {..attributes} node_ref=node_ref as_child=as_child attr:class=class>
            {children()}
        </avatar::AvatarRoot>
    }
}

#[component]
pub fn AvatarImage(
    #[prop(default=(|_|{}).into(), into)] on_loading_status_change: Callback<
        avatar::ImageLoadingStatus,
    >,
    #[prop(into, optional)] class: MaybeSignal<String>,
    #[prop(optional)] node_ref: NodeRef<html::AnyElement>,

    #[prop(optional, into)] as_child: MaybeProp<bool>,
    #[prop(attrs)] attributes: Vec<(&'static str, Attribute)>,
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    let class = create_memo(move |_| tw_merge!("aspect-square h-full w-full", class()));

    let children = StoredValue::new(children);

    view! {
        <avatar::AvatarImage
            {..attributes}
            on_loading_status_change=on_loading_status_change
            node_ref=node_ref
            as_child=as_child
            attr:class=class
        >
            {children.with_value(|children| children.as_ref().map(|children| children()))}
        </avatar::AvatarImage>
    }
}

#[component]
pub fn AvatarFallback(
    #[prop(optional, into)] delay_ms: MaybeSignal<f64>,
    #[prop(optional)] node_ref: NodeRef<html::AnyElement>,
    #[prop(optional, into)] as_child: MaybeProp<bool>,

    #[prop(into, optional)] class: MaybeSignal<String>,
    #[prop(attrs)] attributes: Vec<(&'static str, Attribute)>,
    children: ChildrenFn,
) -> impl IntoView {
    let class = create_memo(move |_| {
        tw_merge!(
            "flex h-full w-full items-center justify-center rounded-full bg-muted",
            class()
        )
    });

    view! {
        <avatar::AvatarFallback
            {..attributes}
            delay_ms=delay_ms
            node_ref=node_ref
            as_child=as_child
            attr:class=class
        >
            {children()}
        </avatar::AvatarFallback>
    }
}
