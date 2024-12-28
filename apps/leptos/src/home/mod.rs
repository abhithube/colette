use feed_entry_list::FeedEntryList;
use leptos::prelude::*;

mod feed_entry_item;
mod feed_entry_list;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div>
            <h1>"Home"</h1>
            <FeedEntryList />
        </div>
    }
}
