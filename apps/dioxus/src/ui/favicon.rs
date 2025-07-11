use dioxus::prelude::*;
use tailwind_fuse::*;
use url::Url;

#[component]
pub fn Favicon(
    class: Option<String>,
    src: Option<String>,
    alt: Option<String>,
    #[props(extends = image)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let domain = src.and_then(|e| {
        e.parse::<Url>()
            .map(|e| e.domain().unwrap_or_default().to_owned())
            .ok()
    });

    let src = domain
        .clone()
        .map(|e| format!("https://icons.duckduckgo.com/ip3/${e}.ico"));
    let alt = alt.or(domain);

    rsx! {
      img {
        class: tw_merge!("size-4", class),
        src,
        alt,
        ..attributes,
        {children}
      }
    }
}
