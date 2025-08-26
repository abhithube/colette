use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_handler::{Handler as _, RefreshFeedCommand, RefreshFeedHandler};
use colette_http::ReqwestClient;
use colette_ingestion::ScrapeFeedJobData;
use colette_queue::Job;
use colette_repository::PostgresFeedRepository;
use futures::FutureExt;
use tower::Service;

use crate::job::Error;

pub struct ScrapeFeedJobHandler {
    refresh_feed: Arc<RefreshFeedHandler<PostgresFeedRepository, ReqwestClient>>,
}

impl ScrapeFeedJobHandler {
    pub fn new(
        refresh_feed: Arc<RefreshFeedHandler<PostgresFeedRepository, ReqwestClient>>,
    ) -> Self {
        Self { refresh_feed }
    }
}

impl Service<Job> for ScrapeFeedJobHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, job: Job) -> Self::Future {
        let refresh_feed = self.refresh_feed.clone();

        async move {
            let data = serde_json::from_value::<ScrapeFeedJobData>(job.data)?;

            tracing::debug!("Scraping feed at URL: {}", data.source_url.as_str());

            refresh_feed
                .handle(RefreshFeedCommand { id: data.feed_id })
                .await
                .map_err(|e| Error::Service(e.to_string()))?;

            Ok(())
        }
        .boxed()
    }
}
