use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::{
    bookmark::{BookmarkRefresh, BookmarkService, ScrapeBookmarkJobData},
    job::Job,
};
use futures::FutureExt;
use tower::Service;

use super::Error;

pub struct ScrapeBookmarkHandler {
    bookmark_service: Arc<BookmarkService>,
}

impl ScrapeBookmarkHandler {
    pub fn new(bookmark_service: Arc<BookmarkService>) -> Self {
        Self { bookmark_service }
    }
}

impl Service<Job> for ScrapeBookmarkHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, job: Job) -> Self::Future {
        let bookmark_service = self.bookmark_service.clone();

        async move {
            let data = serde_json::from_value::<ScrapeBookmarkJobData>(job.data)?;

            tracing::debug!("Scraping bookmark at URL: {}", data.url.as_str());

            bookmark_service
                .refresh_bookmark(BookmarkRefresh {
                    url: data.url,
                    user_id: data.user_id,
                })
                .await
                .map_err(|e| Error::Service(e.to_string()))?;

            Ok(())
        }
        .boxed()
    }
}
