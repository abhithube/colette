use leptix_primitives::label;
use leptos::*;
use tailwind_fuse::*;

#[component]
pub fn Label(
    #[prop(into, optional)] for_html: MaybeProp<String>,
    #[prop(default=(|_|{}).into(), into)] on_mouse_down: Callback<ev::MouseEvent>,
    #[prop(optional)] node_ref: NodeRef<html::AnyElement>,
    #[prop(into, optional)] as_child: MaybeProp<bool>,

    #[prop(into, optional)] class: MaybeSignal<String>,
    #[prop(attrs)] attributes: Vec<(&'static str, Attribute)>,
    children: ChildrenFn,
) -> impl IntoView {
    let class = create_memo(move |_| {
        tw_merge!(
            "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
            class()
        )
    });

    view! {
        <label::LabelRoot
            {..attributes}
            for_html=for_html
            on_mouse_down=on_mouse_down
            node_ref=node_ref
            as_child=as_child
            attr:class=class
        >
            {children()}
        </label::LabelRoot>
    }
}
