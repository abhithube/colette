use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::{
    Handler as _,
    feed::{ListFeedsHandler, ListFeedsQuery, ScrapeFeedJobData},
    job::{CreateJobCommand, CreateJobHandler, Job},
};
use colette_queue::JobProducer;
use futures::FutureExt;
use tokio::sync::Mutex;
use tower::Service;

use crate::{Error, JobError};

pub struct RefreshFeedsJobHandler {
    list_feeds: Arc<ListFeedsHandler>,
    create_job: Arc<CreateJobHandler>,
    scrape_feed_producer: Arc<Mutex<dyn JobProducer>>,
}

impl RefreshFeedsJobHandler {
    pub fn new(
        list_feeds: Arc<ListFeedsHandler>,
        create_job: Arc<CreateJobHandler>,
        scrape_feed_producer: Arc<Mutex<dyn JobProducer>>,
    ) -> Self {
        Self {
            list_feeds,
            create_job,
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

    fn call(&mut self, job: Job) -> Self::Future {
        let list_feeds = self.list_feeds.clone();
        let create_job = self.create_job.clone();
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
                let data = serde_json::to_value(ScrapeFeedJobData {
                    url: feed.source_url,
                })?;

                let job_id = create_job
                    .handle(CreateJobCommand {
                        data,
                        job_type: "scrape_feed".into(),
                        group_identifier: Some(job.id.as_inner().into()),
                    })
                    .await
                    .map_err(JobError::CreateJob)?;

                let mut scrape_feed_producer = scrape_feed_producer.lock().await;

                scrape_feed_producer
                    .push(job_id.as_inner())
                    .await
                    .map_err(|e| Error::Service(e.to_string()))?;
            }

            Ok(())
        }
        .boxed()
    }
}
