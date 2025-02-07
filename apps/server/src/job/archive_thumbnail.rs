use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::bookmark::{
    ArchiveThumbnailJob, BookmarkService, ThumbnailArchive, ThumbnailOperation,
};

pub async fn run(
    job: ArchiveThumbnailJob,
    data: Data<Arc<BookmarkService>>,
) -> Result<(), apalis::prelude::Error> {
    if let ThumbnailOperation::Upload(ref thumbnail_url) = job.operation {
        tracing::debug!("Archiving thumbnail URL: {}", thumbnail_url.as_str());
    }
    if let Some(ref archived_url) = job.archived_url {
        tracing::debug!("Archiving archived URL: {}", archived_url.as_str());
    }

    data.archive_thumbnail(
        job.bookmark_id,
        ThumbnailArchive {
            operation: job.operation,
            archived_url: job.archived_url,
        },
        job.user_id,
    )
    .await
    .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
