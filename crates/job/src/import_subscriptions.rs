use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::{
    feed::ScrapeFeedJobData,
    job::{Job, JobCreate, JobService},
    subscription::{ImportSubscriptionsJobData, SubscriptionListQuery, SubscriptionService},
};
use colette_queue::JobProducer;
use futures::FutureExt;
use tokio::sync::Mutex;
use tower::Service;

use super::Error;

pub struct ImportSubscriptionsHandler {
    subscription_service: Arc<SubscriptionService>,
    job_service: Arc<JobService>,
    scrape_feed_producer: Arc<Mutex<dyn JobProducer>>,
}

impl ImportSubscriptionsHandler {
    pub fn new(
        subscription_service: Arc<SubscriptionService>,
        job_service: Arc<JobService>,
        scrape_feed_producer: Arc<Mutex<dyn JobProducer>>,
    ) -> Self {
        Self {
            subscription_service,
            job_service,
            scrape_feed_producer,
        }
    }
}

impl Service<Job> for ImportSubscriptionsHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, job: Job) -> Self::Future {
        let subscription_service = self.subscription_service.clone();
        let job_service = self.job_service.clone();
        let scrape_feed_producer = self.scrape_feed_producer.clone();

        async move {
            let input_data = serde_json::from_value::<ImportSubscriptionsJobData>(job.data)?;

            let subscriptions = subscription_service
                .list_subscriptions(SubscriptionListQuery::default(), input_data.user_id)
                .await
                .map_err(|e| Error::Service(e.to_string()))?;
            let feeds = subscriptions
                .items
                .into_iter()
                .filter_map(|e| e.feed)
                .collect::<Vec<_>>();

            tracing::debug!("Importing {} feeds", feeds.len());

            for feed in feeds {
                let data = serde_json::to_value(ScrapeFeedJobData {
                    url: feed.source_url,
                })?;

                let job = job_service
                    .create_job(JobCreate {
                        data,
                        job_type: "scrape_feed".into(),
                        group_identifier: Some(job.id.into()),
                    })
                    .await?;

                let mut scrape_feed_producer = scrape_feed_producer.lock().await;

                scrape_feed_producer.push(job.id).await?;
            }

            Ok(())
        }
        .boxed()
    }
}
