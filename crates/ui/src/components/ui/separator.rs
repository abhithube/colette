use leptix_primitives::{separator, Orientation};
use leptos::*;
use tailwind_fuse::*;

#[component]
pub fn Separator(
    #[prop(default = true.into(), into)] decorative: MaybeSignal<bool>,
    #[prop(default = Orientation::Horizontal.into(), into)] orientation: MaybeSignal<Orientation>,
    #[prop(optional)] node_ref: NodeRef<html::AnyElement>,
    #[prop(into, optional)] as_child: MaybeProp<bool>,

    #[prop(into, optional)] class: MaybeSignal<String>,
    #[prop(attrs)] attributes: Vec<(&'static str, Attribute)>,
    #[prop(optional)] children: Option<ChildrenFn>,
) -> impl IntoView {
    let class = create_memo(move |_| {
        tw_merge!(
            "shrink-0 bg-border",
            match orientation() {
                Orientation::Horizontal => "h-[1px] w-full",
                Orientation::Vertical => "h-full w-[1px]",
            },
            class()
        )
    });

    let children = StoredValue::new(children);

    view! {
        <separator::SeparatorRoot
            {..attributes}
            decorative=decorative
            orientation=orientation
            node_ref=node_ref
            as_child=as_child
            attr:class=class
        >
            {children.with_value(|children| children.as_ref().map(|children| children()))}
        </separator::SeparatorRoot>
    }
}
