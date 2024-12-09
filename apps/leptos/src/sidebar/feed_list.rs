use colette_api::feed::{self, ListResponse};
use leptos::prelude::*;

use crate::sidebar::feed_item::FeedItem;

#[component]
pub fn FeedList() -> impl IntoView {
  let feeds_res = Resource::new(
    || (),
    move |_| async move {
        list_feeds().await.map(|e| match e {
            ListResponse::Ok(feeds) => feeds,
        })
    },
);

  view! {
      <ErrorBoundary fallback=|_| ()>
          <Suspense>
              {move || {
                  feeds_res
                      .get()
                      .map(|e| {
                          e.map(|feeds| {
                              feeds
                                  .data
                                  .into_iter()
                                  .map(|feed| view! { <FeedItem feed=feed /> })
                                  .collect_view()
                          })
                      })
              }}
          </Suspense>
      </ErrorBoundary>
  }
}

#[server]
pub async fn list_feeds() -> Result<ListResponse, ServerFnError> {
    use crate::common::auth::validate_session;
    use axum::extract::State;
    use axum_extra::extract::Query;
    use colette_api::{
        feed::{FeedListQuery, FeedState},
        ApiState, Session,
    };
    use colette_core::feed::FeedService;
    use leptos_axum::{extract, extract_with_state};

    let query = extract::<Query<FeedListQuery>>().await?;

    let state = expect_context::<ApiState>();

    let session: Session = validate_session().await?;

    let State(state): State<FeedState> = extract_with_state(&state).await?;
    let state: State<FeedService> = extract_with_state(&state).await?;

    let resp = feed::list_feeds(state, query, session).await?;

    Ok(resp)
}
