use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use colette_crud::{ArchiveThumbnailJobData, ThumbnailOperation};
use colette_handler::{ArchiveThumbnailCommand, ArchiveThumbnailHandler, Handler as _};
use colette_http::ReqwestClient;
use colette_queue::Job;
use colette_repository::PostgresBookmarkRepository;
use colette_s3::S3ClientImpl;
use futures::FutureExt;
use tower::Service;

use crate::Error;

pub struct ArchiveThumbnailJobHandler {
    archive_thumbnail:
        Arc<ArchiveThumbnailHandler<PostgresBookmarkRepository, ReqwestClient, S3ClientImpl>>,
}

impl ArchiveThumbnailJobHandler {
    pub fn new(
        archive_thumbnail: Arc<
            ArchiveThumbnailHandler<PostgresBookmarkRepository, ReqwestClient, S3ClientImpl>,
        >,
    ) -> Self {
        Self { archive_thumbnail }
    }
}

impl Service<Job> for ArchiveThumbnailJobHandler {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, job: Job) -> Self::Future {
        let archive_thumbnail = self.archive_thumbnail.clone();

        async move {
            let data = serde_json::from_value::<ArchiveThumbnailJobData>(job.data)?;

            if let ThumbnailOperation::Upload(ref thumbnail_url) = data.operation {
                tracing::debug!("Archiving thumbnail URL: {}", thumbnail_url.as_str());
            }
            if let Some(ref archived_url) = data.archived_path {
                tracing::debug!("Archiving archived URL: {}", archived_url.as_str());
            }

            archive_thumbnail
                .handle(ArchiveThumbnailCommand {
                    bookmark_id: data.bookmark_id,
                    operation: data.operation,
                    archived_path: data.archived_path,
                })
                .await
                .map_err(|e| Error::Service(e.to_string()))
        }
        .boxed()
    }
}
