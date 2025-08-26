use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_handler::{FetchOutdatedFeedsHandler, FetchOutdatedFeedsQuery, Handler as _};
use colette_ingestion::ScrapeFeedJobData;
use colette_queue::{Job, JobProducer, TokioJobProducer};
use colette_repository::PostgresFeedRepository;
use futures::FutureExt;
use tokio::sync::Mutex;
use tower::Service;

use crate::job::Error;

pub struct RefreshFeedsJobHandler {
    fetch_outdated_feeds: Arc<FetchOutdatedFeedsHandler<PostgresFeedRepository>>,
    scrape_feed_producer: Arc<Mutex<TokioJobProducer>>,
}

impl RefreshFeedsJobHandler {
    pub fn new(
        fetch_outdated_feeds: Arc<FetchOutdatedFeedsHandler<PostgresFeedRepository>>,
        scrape_feed_producer: Arc<Mutex<TokioJobProducer>>,
    ) -> Self {
        Self {
            fetch_outdated_feeds,
            scrape_feed_producer,
        }
    }
}

impl Service<Job> for RefreshFeedsJobHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _job: Job) -> Self::Future {
        let fetch_outdated_feeds = self.fetch_outdated_feeds.clone();
        let scrape_feed_producer = self.scrape_feed_producer.clone();

        async move {
            tracing::debug!("Refreshing feeds");

            let feeds = fetch_outdated_feeds
                .handle(FetchOutdatedFeedsQuery {})
                .await
                .map_err(|e| Error::Service(e.to_string()))?;

            for feed in feeds {
                let data = ScrapeFeedJobData {
                    feed_id: feed.id(),
                    source_url: feed.source_url().to_owned(),
                };
                let job = Job::create("scrape_feed", data)?;

                let mut scrape_feed_producer = scrape_feed_producer.lock().await;

                scrape_feed_producer
                    .push(job)
                    .await
                    .map_err(|e| Error::Service(e.to_string()))?;
            }

            Ok(())
        }
        .boxed()
    }
}
