use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::{
    Handler as _,
    bookmark::{RefreshBookmarkCommand, RefreshBookmarkHandler, ScrapeBookmarkJobData},
    job::Job,
};
use futures::FutureExt;
use tower::Service;

use crate::Error;

pub struct ScrapeBookmarkJobHandler {
    bookmark_service: Arc<RefreshBookmarkHandler>,
}

impl ScrapeBookmarkJobHandler {
    pub fn new(bookmark_service: Arc<RefreshBookmarkHandler>) -> Self {
        Self { bookmark_service }
    }
}

impl Service<Job> for ScrapeBookmarkJobHandler {
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
                .handle(RefreshBookmarkCommand {
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
