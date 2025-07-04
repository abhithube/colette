use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::{
    feed::{FeedService, ScrapeFeedJobData},
    job::{Job, JobCreate, JobService},
};
use colette_queue::JobProducer;
use futures::{FutureExt, StreamExt};
use tokio::sync::Mutex;
use tower::Service;

use super::Error;

pub struct RefreshFeedsHandler {
    feed_service: Arc<FeedService>,
    job_service: Arc<JobService>,
    scrape_feed_producer: Arc<Mutex<dyn JobProducer>>,
}

impl RefreshFeedsHandler {
    pub fn new(
        feed_service: Arc<FeedService>,
        job_service: Arc<JobService>,
        scrape_feed_producer: Arc<Mutex<dyn JobProducer>>,
    ) -> Self {
        Self {
            feed_service,
            job_service,
            scrape_feed_producer,
        }
    }
}

impl Service<Job> for RefreshFeedsHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, job: Job) -> Self::Future {
        let feed_service = self.feed_service.clone();
        let job_service = self.job_service.clone();
        let scrape_feed_producer = self.scrape_feed_producer.clone();

        async move {
            tracing::debug!("Refreshing feeds");

            let mut stream = feed_service
                .stream()
                .await
                .map_err(|e| Error::Service(e.to_string()))?;

            while let Some(url) = stream.next().await {
                let data = serde_json::to_value(ScrapeFeedJobData { url })?;

                let job = job_service
                    .create_job(JobCreate {
                        data,
                        job_type: "scrape_feed".into(),
                        group_identifier: Some(job.id.into()),
                    })
                    .await?;

                let mut scrape_feed_producer = scrape_feed_producer.lock().await;

                scrape_feed_producer
                    .push(job.id)
                    .await
                    .map_err(|e| Error::Service(e.to_string()))?;
            }

            Ok(())
        }
        .boxed()
    }
}
