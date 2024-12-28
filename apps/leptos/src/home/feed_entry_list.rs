use chrono::Utc;
use colette_api::feed_entry::{self, ListResponse};
use leptos::prelude::*;
use timeago::Formatter;

use crate::home::feed_entry_item::FeedEntryItem;

#[component]
pub fn FeedEntryList() -> impl IntoView {
    let feed_entries_res = Resource::new(
        || (),
        move |_| async move {
            list_feed_entries().await.map(|e| match e {
                ListResponse::Ok(feed_entries) => feed_entries,
            })
        },
    );

    let now = RwSignal::new(Utc::now());
    let formatter = RwSignal::new(Formatter::new());

    view! {
        <Suspense>
            {move || {
                feed_entries_res
                    .get()
                    .map(|e| {
                        e.map(|feed_entries| {
                            feed_entries
                                .data
                                .into_iter()
                                .map(|feed_entry| {
                                    view! {
                                        <FeedEntryItem
                                            feed_entry=feed_entry
                                            format=move |date| {
                                                formatter.read().convert_chrono(date, now.get())
                                            }
                                        />
                                    }
                                })
                                .collect_view()
                        })
                    })
            }}
        </Suspense>
    }
}

#[server]
pub async fn list_feed_entries() -> Result<ListResponse, ServerFnError> {
    use crate::common::auth::validate_session;
    use axum::extract::State;
    use axum_extra::extract::Query;
    use colette_api::{
        feed_entry::{FeedEntryListQuery, FeedEntryState},
        ApiState, Session,
    };
    use colette_core::feed_entry::FeedEntryService;
    use leptos_axum::{extract, extract_with_state};

    let query = extract::<Query<FeedEntryListQuery>>().await?;

    let state = expect_context::<ApiState>();

    let session: Session = validate_session().await?;

    let State(state): State<FeedEntryState> = extract_with_state(&state).await?;
    let state: State<FeedEntryService> = extract_with_state(&state).await?;

    let resp = feed_entry::list_feed_entries(state, query, session).await?;

    Ok(resp)
}
