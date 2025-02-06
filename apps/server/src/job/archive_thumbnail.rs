use std::sync::Arc;

use apalis::prelude::Data;
use colette_core::bookmark::{ArchiveThumbnailJob, BookmarkService, ThumbnailArchive};

pub async fn run(
    job: ArchiveThumbnailJob,
    data: Data<Arc<BookmarkService>>,
) -> Result<(), apalis::prelude::Error> {
    tracing::debug!("Archiving thumbnail URL: {}", job.url.as_str());

    data.archive_thumbnail(
        job.bookmark_id,
        ThumbnailArchive {
            thumbnail_url: job.url,
        },
        job.user_id,
    )
    .await
    .map_err(|e| apalis::prelude::Error::Failed(Arc::new(Box::new(e))))?;

    Ok(())
}
