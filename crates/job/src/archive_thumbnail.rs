use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_core::{
    bookmark::{ArchiveThumbnailJobData, BookmarkService, ThumbnailArchive, ThumbnailOperation},
    job::Job,
};
use futures::FutureExt;
use tower::Service;

use super::Error;

pub struct ArchiveThumbnailHandler {
    bookmark_service: Arc<BookmarkService>,
}

impl ArchiveThumbnailHandler {
    pub fn new(service: Arc<BookmarkService>) -> Self {
        Self {
            bookmark_service: service,
        }
    }
}

impl Service<Job> for ArchiveThumbnailHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, job: Job) -> Self::Future {
        let service = self.bookmark_service.clone();

        async move {
            let data = serde_json::from_value::<ArchiveThumbnailJobData>(job.data)?;

            if let ThumbnailOperation::Upload(ref thumbnail_url) = data.operation {
                tracing::debug!("Archiving thumbnail URL: {}", thumbnail_url.as_str());
            }
            if let Some(ref archived_url) = data.archived_path {
                tracing::debug!("Archiving archived URL: {}", archived_url.as_str());
            }

            service
                .archive_thumbnail(
                    data.bookmark_id,
                    ThumbnailArchive {
                        operation: data.operation,
                        archived_path: data.archived_path,
                    },
                )
                .await
                .map_err(|e| Error::Service(e.to_string()))
        }
        .boxed()
    }
}
