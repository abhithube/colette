use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::feed::ScrapeFeedJobData;
use colette_handler::{Handler as _, ListFeedsHandler, ListFeedsQuery};
use colette_queue::{Job, JobProducer, TokioJobProducer};
use colette_repository::PostgresFeedRepository;
use futures::FutureExt;
use tokio::sync::Mutex;
use tower::Service;

use crate::Error;

pub struct RefreshFeedsJobHandler {
    list_feeds: Arc<ListFeedsHandler<PostgresFeedRepository>>,
    scrape_feed_producer: Arc<Mutex<TokioJobProducer>>,
}

impl RefreshFeedsJobHandler {
    pub fn new(
        list_feeds: Arc<ListFeedsHandler<PostgresFeedRepository>>,
        scrape_feed_producer: Arc<Mutex<TokioJobProducer>>,
    ) -> Self {
        Self {
            list_feeds,
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
        let list_feeds = self.list_feeds.clone();
        let scrape_feed_producer = self.scrape_feed_producer.clone();

        async move {
            tracing::debug!("Refreshing feeds");

            let feeds = list_feeds
                .handle(ListFeedsQuery {
                    limit: Some(100),
                    ready_to_refresh: true,
                    ..Default::default()
                })
                .await
                .map_err(|e| Error::Service(e.to_string()))?;

            for feed in feeds.items {
                let data = ScrapeFeedJobData {
                    url: feed.source_url,
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
