use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::{
    Handler as _,
    bookmark::{
        ImportBookmarksJobData, ListBookmarksHandler, ListBookmarksQuery, ScrapeBookmarkJobData,
    },
};
use colette_queue::{Job, JobProducer, TokioJobProducer};
use colette_repository::{PostgresBookmarkRepository, PostgresCollectionRepository};
use futures::FutureExt;
use tokio::sync::Mutex;
use tower::Service;

use crate::Error;

pub struct ImportBookmarksJobHandler {
    list_bookmarks:
        Arc<ListBookmarksHandler<PostgresBookmarkRepository, PostgresCollectionRepository>>,
    scrape_bookmark_producer: Arc<Mutex<TokioJobProducer>>,
}

impl ImportBookmarksJobHandler {
    pub fn new(
        list_bookmarks: Arc<
            ListBookmarksHandler<PostgresBookmarkRepository, PostgresCollectionRepository>,
        >,
        scrape_bookmark_producer: Arc<Mutex<TokioJobProducer>>,
    ) -> Self {
        Self {
            list_bookmarks,
            scrape_bookmark_producer,
        }
    }
}

impl Service<Job> for ImportBookmarksJobHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, job: Job) -> Self::Future {
        let list_bookmarks = self.list_bookmarks.clone();
        let scrape_bookmark_producer = self.scrape_bookmark_producer.clone();

        async move {
            let input_data = serde_json::from_value::<ImportBookmarksJobData>(job.data)?;

            let bookmarks = list_bookmarks
                .handle(ListBookmarksQuery {
                    collection_id: None,
                    tags: None,
                    cursor: None,
                    limit: None,
                    user_id: input_data.user_id,
                })
                .await
                .map_err(|e| Error::Service(e.to_string()))?;

            tracing::debug!("Importing {} bookmarks", bookmarks.items.len());

            for bookmark in bookmarks.items {
                let data = ScrapeBookmarkJobData {
                    url: bookmark.link,
                    user_id: input_data.user_id,
                };
                let job = Job::create("scrape_bookmark", data)?;

                let mut scrape_bookmark_producer = scrape_bookmark_producer.lock().await;

                scrape_bookmark_producer.push(job).await?;
            }

            Ok(())
        }
        .boxed()
    }
}
