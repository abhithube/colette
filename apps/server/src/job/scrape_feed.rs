use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::{
    feed::{FeedScrape, FeedService, ScrapeFeedJobData},
    job::Job,
};
use futures::FutureExt;
use tower::Service;

use super::Error;

pub struct ScrapeFeedHandler {
    feed_service: Arc<FeedService>,
}

impl ScrapeFeedHandler {
    pub fn new(feed_service: Arc<FeedService>) -> Self {
        Self { feed_service }
    }
}

impl Service<Job> for ScrapeFeedHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, job: Job) -> Self::Future {
        let feed_service = self.feed_service.clone();

        async move {
            let data = serde_json::from_value::<ScrapeFeedJobData>(job.data)?;

            tracing::debug!("Scraping feed at URL: {}", data.url.as_str());

            feed_service
                .scrape_feed(FeedScrape { url: data.url })
                .await
                .map_err(|e| Error::Service(e.to_string()))?;

            Ok(())
        }
        .boxed()
    }
}
