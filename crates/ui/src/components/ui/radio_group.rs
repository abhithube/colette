use leptix_primitives::{radio_group, Direction, Orientation};
use leptos::*;
use tailwind_fuse::*;

use crate::components::ui::icons::Circle;

#[component]
pub fn Root(
    #[prop(into, optional)] name: MaybeProp<String>,
    #[prop(into, optional)] value: MaybeProp<String>,
    #[prop(into, optional)] default_value: MaybeProp<String>,
    #[prop(into, optional)] required: MaybeSignal<bool>,
    #[prop(into, optional)] disabled: MaybeSignal<bool>,
    #[prop(into, optional)] should_loop: MaybeSignal<bool>,
    #[prop(into, optional)] direction: MaybeSignal<Direction>,
    #[prop(into, optional)] orientation: MaybeSignal<Orientation>,
    #[prop(default=(|_|{}).into(), into)] on_value_change: Callback<String>,
    #[prop(optional)] node_ref: NodeRef<html::AnyElement>,
    #[prop(optional, into)] as_child: MaybeProp<bool>,

    #[prop(into, optional)] class: MaybeSignal<String>,
    #[prop(attrs)] attributes: Vec<(&'static str, Attribute)>,
    children: ChildrenFn,
) -> impl IntoView {
    let class = create_memo(move |_| tw_merge!("grid gap-2", class()));

    view! {
        <radio_group::RadioGroupRoot
            {..attributes}
            name=name
            value=value
            default_value=default_value
            required=required
            disabled=disabled
            should_loop=should_loop
            direction=direction
            orientation=orientation
            on_value_change=on_value_change
            node_ref=node_ref
            as_child=as_child
            attr:class=class
        >
            {children()}
        </radio_group::RadioGroupRoot>
    }
}

#[component]
pub fn Item(
    #[prop(into)] value: MaybeSignal<String>,
    #[prop(into, optional)] disabled: MaybeSignal<bool>,
    #[prop(default=(|_|{}).into(), into)] on_focus: Callback<ev::FocusEvent>,
    #[prop(default=(|_|{}).into(), into)] on_key_down: Callback<ev::KeyboardEvent>,
    #[prop(into, optional)] class: MaybeSignal<String>,
    #[prop(optional)] node_ref: NodeRef<html::AnyElement>,

    #[prop(optional, into)] as_child: MaybeProp<bool>,
    #[prop(attrs)] attributes: Vec<(&'static str, Attribute)>,
    // children: ChildrenFn,
) -> impl IntoView {
    let class = create_memo(move |_| {
        tw_merge!("aspect-square h-4 w-4 rounded-full border border-primary text-primary ring-offset-background focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50", class())
    });

    // let children = StoredValue::new(children);

    view! {
        <radio_group::RadioGroupItem
            {..attributes}
            value=value
            disabled=disabled
            on_focus=on_focus
            on_key_down=on_key_down
            node_ref=node_ref
            as_child=as_child
            attr:class=class
        >
            <radio_group::RadioGroupIndicator attr:class="flex items-center justify-center">
                <Circle attr:class="h-2.5 w-2.5 fill-current text-current" />
            </radio_group::RadioGroupIndicator>
        // {children.with_value(|children| children())}
        </radio_group::RadioGroupItem>
    }
}
