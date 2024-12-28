use crate::common::{icons::ExternalLink, ui::card};
use chrono::{DateTime, Utc};
use colette_api::feed_entry::FeedEntry;
use leptos::prelude::*;
use url::Url;

#[component]
pub fn FeedEntryItem(
    feed_entry: FeedEntry,
    format: impl Fn(DateTime<Utc>) -> String + Send + 'static,
) -> impl IntoView {
    let domain = Url::parse(&feed_entry.link)
        .unwrap()
        .domain()
        .unwrap()
        .to_owned();

    let thumbnail_url = feed_entry
        .thumbnail_url
        .unwrap_or_else(|| "https://placehold.co/320x180/black/black".to_owned());
    let description = feed_entry
        .description
        .unwrap_or_else(|| "No description".to_owned());

    view! {
        <card::Root class="flex">
            <div class="aspect-[16/9] flex-shrink-0">
                <img
                    src=thumbnail_url
                    alt=feed_entry.title.clone()
                    class="w-full h-full object-cover"
                />
            </div>
            <div class="flex flex-col">
                <card::Header>
                    <card::Title class="truncate">{feed_entry.title}</card::Title>
                </card::Header>
                <card::Content>
                    <card::Description class="line-clamp-2">{description}</card::Description>
                </card::Content>

                <card::Footer class="justify-between">
                    <div class="flex items-center gap-2 text-sm">
                        <img
                            class="size-5 shrink-0"
                            src=format!("https://icons.duckduckgo.com/ip3/{}.ico", domain)
                            width=16
                            height=16
                            alt=domain
                        />
                        <span class="truncate">{feed_entry.author}</span>
                        <span>{format(feed_entry.published_at)}</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <a href=feed_entry.link target="_blank">
                            <ExternalLink />
                        </a>
                        <input type="checkbox" />
                    </div>
                </card::Footer>
            </div>
        </card::Root>
    }
}
