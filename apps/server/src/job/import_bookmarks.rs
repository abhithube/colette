use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::{
    backup::ImportBookmarksJobData,
    bookmark::{BookmarkListQuery, BookmarkService, ScrapeBookmarkJobData},
    job::{Job, JobCreate, JobService},
};
use colette_queue::JobProducer;
use futures::FutureExt;
use tokio::sync::Mutex;
use tower::Service;

use super::Error;

pub struct ImportBookmarksHandler {
    bookmark_service: Arc<BookmarkService>,
    job_service: Arc<JobService>,
    scrape_bookmark_producer: Arc<Mutex<dyn JobProducer>>,
}

impl ImportBookmarksHandler {
    pub fn new(
        bookmark_service: Arc<BookmarkService>,
        job_service: Arc<JobService>,
        scrape_bookmark_producer: Arc<Mutex<dyn JobProducer>>,
    ) -> Self {
        Self {
            bookmark_service,
            job_service,
            scrape_bookmark_producer,
        }
    }
}

impl Service<Job> for ImportBookmarksHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, job: Job) -> Self::Future {
        let bookmark_service = self.bookmark_service.clone();
        let job_service = self.job_service.clone();
        let scrape_bookmark_producer = self.scrape_bookmark_producer.clone();

        async move {
            let input_data = serde_json::from_value::<ImportBookmarksJobData>(job.data)?;

            let bookmarks = bookmark_service
                .list_bookmarks(BookmarkListQuery::default(), input_data.user_id)
                .await
                .map_err(|e| Error::Service(e.to_string()))?;

            tracing::debug!("Importing {} bookmarks", bookmarks.data.len());

            for bookmark in bookmarks.data {
                let data = serde_json::to_value(ScrapeBookmarkJobData {
                    url: bookmark.link,
                    user_id: bookmark.user_id,
                })?;

                let job = job_service
                    .create_job(JobCreate {
                        data,
                        job_type: "scrape_bookmark".into(),
                        group_id: Some(job.id.into()),
                    })
                    .await?;

                let mut scrape_bookmark_producer = scrape_bookmark_producer.lock().await;

                scrape_bookmark_producer.push(job.id).await?;
            }

            Ok(())
        }
        .boxed()
    }
}
