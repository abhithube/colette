use colette_api::feed::Feed;
use leptos::prelude::*;
use leptos_router::components::A;
use url::Url;

#[component]
pub fn FeedItem(feed: Feed) -> impl IntoView {
    let domain = Url::parse(&feed.link).unwrap().domain().unwrap().to_owned();
    let title = feed.title.unwrap_or(feed.original_title);

    let tip = title.clone();

    view! {
        <li>
            <A
                href=format!("/feeds/{}", feed.id)
                {..}
                class="flex items-center tooltip tooltip-right"
                data-tip=tip
            >
                <img
                    class="size-5 shrink-0"
                    src=format!("https://icons.duckduckgo.com/ip3/{}.ico", domain)
                    width=16
                    height=16
                    alt=domain
                />
                <span class="truncate">{title}</span>
            </A>
        </li>
    }
}
