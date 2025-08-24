use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::bookmark::ScrapeBookmarkJobData;
use colette_handler::{Handler as _, RefreshBookmarkCommand, RefreshBookmarkHandler};
use colette_http::ReqwestClient;
use colette_queue::Job;
use colette_repository::PostgresBookmarkRepository;
use futures::FutureExt;
use tower::Service;

use crate::Error;

pub struct ScrapeBookmarkJobHandler {
    refresh_bookmark: Arc<RefreshBookmarkHandler<PostgresBookmarkRepository, ReqwestClient>>,
}

impl ScrapeBookmarkJobHandler {
    pub fn new(
        refresh_bookmark: Arc<RefreshBookmarkHandler<PostgresBookmarkRepository, ReqwestClient>>,
    ) -> Self {
        Self { refresh_bookmark }
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
        let refresh_bookmark = self.refresh_bookmark.clone();

        async move {
            let data = serde_json::from_value::<ScrapeBookmarkJobData>(job.data)?;

            tracing::debug!("Scraping bookmark at URL: {}", data.url.as_str());

            refresh_bookmark
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
